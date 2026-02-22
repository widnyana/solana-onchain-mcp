use solana_sdk::pubkey::Pubkey;

use crate::error::{Result, SolanaMcpError};

pub trait ParsePubkeyExt {
    fn parse_pubkey(&self) -> Result<Pubkey>;
}

impl ParsePubkeyExt for str {
    fn parse_pubkey(&self) -> Result<Pubkey> {
        self.parse::<Pubkey>()
            .map_err(|e| SolanaMcpError::InvalidAddress(e.to_string()))
    }
}
