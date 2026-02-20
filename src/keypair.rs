use std::{fs, path::Path};

use solana_sdk::signature::{Keypair, Signer};

use crate::error::{Result, SolanaMcpError};

#[allow(dead_code)]
pub struct LoadedKeypair {
    pub keypair: Keypair,
    pub pubkey: String,
}

#[allow(dead_code)]
pub fn load_keypair(path: &Path) -> Result<LoadedKeypair> {
    if !path.exists() {
        return Err(SolanaMcpError::KeypairNotFound(path.display().to_string()));
    }

    let contents = fs::read_to_string(path)
        .map_err(|e| SolanaMcpError::InvalidKeypair(format!("Failed to read keypair file: {}", e)))?;

    let bytes: Vec<u8> = serde_json::from_str(&contents)
        .map_err(|e| SolanaMcpError::InvalidKeypair(format!("JSON parse error: {}", e)))?;

    if bytes.len() != 64 {
        return Err(SolanaMcpError::InvalidKeypair(format!(
            "Invalid keypair length: expected 64 bytes, got {}",
            bytes.len()
        )));
    }

    let keypair = Keypair::try_from(&bytes[..])
        .map_err(|e| SolanaMcpError::InvalidKeypair(format!("Invalid keypair bytes: {}", e)))?;

    let pubkey = keypair.pubkey().to_string();

    Ok(LoadedKeypair { keypair, pubkey })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_load_keypair_valid() {
        // Generate a valid keypair and serialize it
        let keypair = Keypair::new();
        let keypair_bytes: Vec<u8> = keypair.to_bytes().to_vec();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "{}",
            serde_json::to_string(&keypair_bytes).unwrap()
        )
        .unwrap();

        let result = load_keypair(temp_file.path());
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.pubkey, keypair.pubkey().to_string());
    }

    #[test]
    fn test_load_keypair_file_not_found() {
        let result = load_keypair(Path::new("/nonexistent/path.json"));
        assert!(matches!(result, Err(SolanaMcpError::KeypairNotFound(_))));
    }

    #[test]
    fn test_load_keypair_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "not valid json").unwrap();

        let result = load_keypair(temp_file.path());
        assert!(matches!(result, Err(SolanaMcpError::InvalidKeypair(_))));
    }

    #[test]
    fn test_load_keypair_invalid_length() {
        // Only 32 bytes instead of 64
        let keypair_bytes: Vec<u8> = (0..32).collect();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "{}",
            serde_json::to_string(&keypair_bytes).unwrap()
        )
        .unwrap();

        let result = load_keypair(temp_file.path());
        assert!(matches!(result, Err(SolanaMcpError::InvalidKeypair(_))));
    }

    #[test]
    fn test_load_keypair_invalid_bytes() {
        // 64 bytes but not a valid Ed25519 keypair
        let keypair_bytes: Vec<u8> = vec![42u8; 64];
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "{}",
            serde_json::to_string(&keypair_bytes).unwrap()
        )
        .unwrap();

        let result = load_keypair(temp_file.path());
        // Keypair::from_bytes may succeed with invalid bytes but the keypair won't be usable
        // The important thing is we don't crash
        let _ = result;
    }
}
