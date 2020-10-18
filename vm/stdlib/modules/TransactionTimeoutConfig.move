address 0x1 {

module TransactionTimeoutConfig {
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Config;

    struct TransactionTimeoutConfig {
        duration_seconds: u64,
    }

    public fun initialize(account: &signer, duration_seconds: u64) {
        assert(Timestamp::is_genesis(), Errors::invalid_state(Errors::ENOT_GENESIS()));
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), Errors::requires_address(Errors::ENOT_GENESIS_ACCOUNT()));

        Config::publish_new_config<Self::TransactionTimeoutConfig>(
            account,
            new_transaction_timeout_config(duration_seconds)
        );
    }

    public fun new_transaction_timeout_config(duration_seconds: u64) : TransactionTimeoutConfig {
        TransactionTimeoutConfig {duration_seconds: duration_seconds}
    }

    public fun get_transaction_timeout_config(): TransactionTimeoutConfig {
        Config::get_by_address<TransactionTimeoutConfig>(CoreAddresses::GENESIS_ADDRESS())
    }

    public fun duration_seconds() :u64 {
        let config = get_transaction_timeout_config();
        config.duration_seconds
    }
}
}