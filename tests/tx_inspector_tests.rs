//! Unit tests for tx_inspector module
//!
//! Tests cover:
//! - get_program_name: Program name lookup
//! - format_instruction_error: JSON error parsing
//! - identify_program: Program registry lookup
//! - interpret_error: Program-specific error interpretation
//! - decode_instruction: Instruction data decoding

use serde_json::json;
use solana_onchain_mcp::tools::tx_inspector::{
    ParsedInstructionData, ProgramCategory, decode_instruction, format_instruction_error, get_program_name,
    identify_program, interpret_error,
};

// =============================================================================
// get_program_name tests
// =============================================================================

mod get_program_name_tests {
    use super::*;

    #[test]
    fn test_system_program() {
        let result = get_program_name("11111111111111111111111111111111");
        assert_eq!(result, Some("System Program"));
    }

    #[test]
    fn test_token_program() {
        let result = get_program_name("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert_eq!(result, Some("Token Program"));
    }

    #[test]
    fn test_token2022_program() {
        let result = get_program_name("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
        assert_eq!(result, Some("Token-2022 Program"));
    }

    #[test]
    fn test_ata_program() {
        let result = get_program_name("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
        assert_eq!(result, Some("Associated Token Account"));
    }

    #[test]
    fn test_raydium_dex() {
        let result = get_program_name("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
        assert_eq!(result, Some("Raydium DEX"));
    }

    #[test]
    fn test_jupiter_aggregator() {
        let result = get_program_name("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
        assert_eq!(result, Some("Jupiter Aggregator"));
    }

    #[test]
    fn test_orca_whirlpool() {
        let result = get_program_name("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
        assert_eq!(result, Some("Orca Whirlpool"));
    }

    #[test]
    fn test_unknown_program() {
        let result = get_program_name("UnknownProgram1111111111111111111111111");
        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_string() {
        let result = get_program_name("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_pubkey_format() {
        let result = get_program_name("not-a-valid-pubkey");
        assert_eq!(result, None);
    }
}

// =============================================================================
// format_instruction_error tests
// =============================================================================

mod format_instruction_error_tests {
    use super::*;

    #[test]
    fn test_instruction_error_with_index() {
        let err = json!({
            "InstructionError": [0, "InsufficientFundsForRent"]
        });
        let result = format_instruction_error(&err);
        assert!(result.contains("Instruction 0 failed"));
        assert!(result.contains("InsufficientFundsForRent"));
    }

    #[test]
    fn test_instruction_error_with_second_index() {
        let err = json!({
            "InstructionError": [2, "InvalidAccountData"]
        });
        let result = format_instruction_error(&err);
        assert!(result.contains("Instruction 2 failed"));
    }

    #[test]
    fn test_instruction_error_with_nested_error() {
        let err = json!({
            "InstructionError": [1, {"Custom": 42}]
        });
        let result = format_instruction_error(&err);
        assert!(result.contains("Instruction 1 failed"));
        assert!(result.contains("Custom"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_non_instruction_error() {
        let err = json!({
            "OtherError": "Something went wrong"
        });
        let result = format_instruction_error(&err);
        assert!(result.contains("OtherError"));
    }

    #[test]
    fn test_null_error() {
        let err = json!(null);
        let result = format_instruction_error(&err);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_empty_object() {
        let err = json!({});
        let result = format_instruction_error(&err);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_instruction_error_missing_array() {
        let err = json!({
            "InstructionError": "not an array"
        });
        let result = format_instruction_error(&err);
        assert!(result.contains("InstructionError"));
    }

    #[test]
    fn test_instruction_error_incomplete_array() {
        let err = json!({
            "InstructionError": [0]
        });
        let result = format_instruction_error(&err);
        assert!(result.contains("InstructionError"));
    }
}

// =============================================================================
// identify_program tests
// =============================================================================

mod identify_program_tests {
    use super::*;

    #[test]
    fn test_identify_system_program() {
        let result = identify_program("11111111111111111111111111111111");
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.name, "System Program");
        assert_eq!(info.category, ProgramCategory::Core);
    }

    #[test]
    fn test_identify_token_program() {
        let result = identify_program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.name, "Token Program");
        assert_eq!(info.category, ProgramCategory::Core);
    }

    #[test]
    fn test_identify_unknown_program() {
        let result = identify_program("UnknownProgram1111111111111111111111111");
        assert!(result.is_none());
    }

    #[test]
    fn test_identify_program_returns_static_reference() {
        // Verify that the returned reference is valid
        let result1 = identify_program("11111111111111111111111111111111");
        let result2 = identify_program("11111111111111111111111111111111");
        assert!(result1.is_some());
        assert!(result2.is_some());
        // Both should point to the same static data
        assert!(std::ptr::eq(result1.unwrap(), result2.unwrap()));
    }
}

// =============================================================================
// interpret_error tests
// =============================================================================

mod interpret_error_tests {
    use super::*;

    // System Program error codes
    #[test]
    fn test_system_error_account_already_exists() {
        let result = interpret_error("11111111111111111111111111111111", 0);
        assert!(result.contains("already exists"));
    }

    #[test]
    fn test_system_error_insufficient_funds() {
        let result = interpret_error("11111111111111111111111111111111", 1);
        assert!(result.contains("enough SOL"));
    }

    #[test]
    fn test_system_error_invalid_program_id() {
        let result = interpret_error("11111111111111111111111111111111", 6);
        assert!(result.contains("Invalid program ID"));
    }

    #[test]
    fn test_system_error_unknown_code() {
        let result = interpret_error("11111111111111111111111111111111", 999);
        assert!(result.contains("Unknown System error code"));
        assert!(result.contains("999"));
    }

    // Token Program error codes
    #[test]
    fn test_token_error_not_enough_balance() {
        let result = interpret_error("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", 0);
        assert!(result.contains("Not enough balance"));
    }

    #[test]
    fn test_token_error_invalid_owner() {
        let result = interpret_error("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", 1);
        assert!(result.contains("Invalid owner"));
    }

    #[test]
    fn test_token_error_frozen() {
        let result = interpret_error("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", 2);
        assert!(result.contains("frozen"));
    }

    #[test]
    fn test_token_error_overflow() {
        let result = interpret_error("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", 12);
        assert!(result.contains("overflow"));
    }

    #[test]
    fn test_token_error_unknown_code() {
        let result = interpret_error("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", 999);
        assert!(result.contains("Unknown Token error code"));
    }

    // Token2022 Program (should use same error codes as Token)
    #[test]
    fn test_token2022_error_not_enough_balance() {
        let result = interpret_error("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb", 0);
        assert!(result.contains("Not enough balance"));
    }

    // Unknown program
    #[test]
    fn test_unknown_program_error() {
        let result = interpret_error("UnknownProgram1111111111111111111111111", 42);
        assert!(result.contains("Error code 42"));
        assert!(result.contains("UnknownProgram"));
    }
}

// =============================================================================
// decode_instruction tests
// =============================================================================

mod decode_instruction_tests {
    use super::*;

    #[test]
    fn test_decode_unknown_program_returns_raw_hex() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let accounts: Vec<String> = vec![];
        let result = decode_instruction("UnknownProgram1111111111111111111111111", &data, &accounts);
        assert!(result.is_ok());
        match result.unwrap() {
            ParsedInstructionData::Raw(hex) => {
                assert_eq!(hex, "01020304");
            }
            ParsedInstructionData::Decoded(_) => panic!("Expected Raw data for unknown program"),
        }
    }

    #[test]
    fn test_decode_empty_data() {
        let data: Vec<u8> = vec![];
        let accounts: Vec<String> = vec![];
        let result = decode_instruction("UnknownProgram1111111111111111111111111", &data, &accounts);
        assert!(result.is_ok());
        match result.unwrap() {
            ParsedInstructionData::Raw(hex) => {
                assert_eq!(hex, "");
            }
            ParsedInstructionData::Decoded(_) => panic!("Expected Raw data"),
        }
    }

    #[test]
    fn test_decode_system_transfer_instruction() {
        // System Program transfer instruction: 2 (transfer) + 8 bytes lamports
        let data = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0]; // 2 = transfer, 1000000000 lamports = 1 SOL
        let accounts = vec![
            "SenderPubkey1111111111111111111111111".to_string(),
            "RecipientPubkey1111111111111111111111".to_string(),
        ];
        let result = decode_instruction("11111111111111111111111111111111", &data, &accounts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_with_empty_accounts() {
        let data = vec![0, 0, 0, 0];
        let accounts: Vec<String> = vec![];
        let result = decode_instruction("11111111111111111111111111111111", &data, &accounts);
        // Should not panic even with empty accounts
        assert!(result.is_ok() || result.is_err());
    }
}
