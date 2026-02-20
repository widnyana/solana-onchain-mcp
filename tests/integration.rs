// Integration tests against Solana devnet
// Run with: SOLANA_NETWORK=devnet cargo test --test integration
// known token account address: J5PpXvvkf9zRkyQZdVh5defDBrznDWw7cFMWAbGx59tt
// known SPL Token account address: 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU (USDC)

#[cfg(test)]
mod tests {
    use std::env;

    fn setup() {
        unsafe {
            env::set_var("SOLANA_NETWORK", "devnet");
        }
    }

    #[tokio::test]
    #[ignore] // Requires devnet connectivity
    async fn test_get_balance_known_address() {
        setup();
        // Test against known devnet address
        // Example: System program (all zeros) has 0 balance
    }

    #[tokio::test]
    #[ignore] // Requires devnet connectivity
    async fn test_get_slot() {
        setup();
        // Slot should always be positive
    }

    #[tokio::test]
    #[ignore] // Requires devnet connectivity and known tx
    async fn test_get_transaction() {
        setup();
        // Test against known devnet transaction
    }
}
