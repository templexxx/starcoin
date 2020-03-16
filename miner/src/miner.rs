// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use chain::ChainActor;
use std::sync::Arc;
use consensus::{ConsensusHeader, dummy::DummyHeader, consensus_impl, Consensus};
use futures::channel::oneshot;
use logger::prelude::*;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::convert::TryFrom;
use traits::ChainReader;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction, block::BlockTemplate};
use futures::channel::mpsc;
use futures::AsyncReadExt;
use futures::{Future, TryFutureExt};
use futures::prelude::*;
use sc_stratum::*;

#[derive(Clone)]
pub struct Miner {
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: Addr<BusActor>,
}

#[derive(Clone)]
pub struct MineCtx {
    header_hash: Vec<u8>,
    block_template: BlockTemplate,
}


impl MineCtx {
    pub fn new(block_template: BlockTemplate) -> MineCtx {
        let header_hash = block_template.clone().into_block_header(DummyHeader {}).id().to_vec();
        MineCtx {
            header_hash,
            block_template,
        }
    }
}

pub fn mint<C>(
    config: Arc<NodeConfig>,
    txns: Vec<SignedUserTransaction>,
    chain: &dyn ChainReader,
    bus: Addr<BusActor>,
) -> Result<()>
    where
        C: Consensus,
{
    let block_template = chain.create_block_template(txns)?;
    let (_sender, receiver) = oneshot::channel();
    // spawn a async task, maintain a task list, when new task coming, cancel old task.
    let block = C::create_block(config, chain, block_template, receiver)?;
    // fire SystemEvents::MinedBlock.
    //TODO handle result.
    info!("broadcast new block: {:?}.", block.header().id());
    bus.do_send(Broadcast {
        msg: SystemEvents::MinedBlock(block),
    });
    Ok(())
}

impl Miner {
    pub fn new(bus: Addr<BusActor>) -> Miner {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
        }
    }
    pub fn set_mint_job(&mut self, t: MineCtx) {
        let mut state = self.state.lock().unwrap();
        *state = Some(t)
    }

    pub fn get_mint_job(&mut self) -> String {
        let mut state = self.state.lock().unwrap();
        let x = state.as_ref().unwrap().to_owned();
        "".to_ascii_lowercase()
    }

    pub fn submit(&self, payload: Vec<u8>) {
        // verify payload
        // create block
        let state = self.state.lock().unwrap();
        let block_template = state.as_ref().unwrap().block_template.clone();
        let consensus_header = consensus_impl::ConsensusHeaderImpl::try_from(payload).unwrap();
        let block = block_template.into_block(consensus_header);
        // notify chain mined block
        println!("miner new block: {:?}", block);
        ///fire SystemEvents::MinedBlock.
        info!("broadcast new block: {:?}.", block.header().id());
        self.bus.do_send(Broadcast {
            msg: SystemEvents::MinedBlock(block),
        });
    }
}