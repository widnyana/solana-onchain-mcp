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
        assert_eq!(result.as_deref(), Some("System Program"));
    }

    #[test]
    fn test_token_program() {
        let result = get_program_name("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert_eq!(result.as_deref(), Some("Token Program"));
    }

    #[test]
    fn test_token2022_program() {
        let result = get_program_name("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
        assert_eq!(result.as_deref(), Some("Token-2022 Program"));
    }

    #[test]
    fn test_ata_program() {
        let result = get_program_name("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
        assert_eq!(result.as_deref(), Some("Associated Token Account"));
    }

    #[test]
    fn test_raydium_dex() {
        let result = get_program_name("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
        assert_eq!(result.as_deref(), Some("Raydium DEX"));
    }

    #[test]
    fn test_jupiter_aggregator() {
        let result = get_program_name("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
        assert_eq!(result.as_deref(), Some("Jupiter Aggregator"));
    }

    #[test]
    fn test_orca_whirlpool() {
        let result = get_program_name("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
        assert_eq!(result.as_deref(), Some("Orca Whirlpool"));
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

// =============================================================================
// humanize_transaction_to_json output-contract tests
// =============================================================================

mod humanize_transaction_to_json_tests {
    use serde_json::json;
    use solana_onchain_mcp::tools::tx_inspector::humanize_transaction_to_json;

    #[test]
    fn test_empty_input_returns_safe_defaults() {
        let result = humanize_transaction_to_json(&json!({}));
        assert!(!result.is_null());
        assert_eq!(result["signature"].as_str(), Some("unknown"));
        assert_eq!(result["fee"].as_u64(), Some(0));
    }

    #[test]
    fn test_valid_input_has_required_keys() {
        let tx = json!({
            "slot": 12345,
            "blockTime": 1700000000i64,
            "meta": {"fee": 5000, "err": null},
            "transaction": {
                "signatures": ["4rFeTz3SUfDZbNKaMYQ6tD5T7YinxhZpGpSfVKeMBrMV12345"],
                "message": {
                    "accountKeys": [
                        {"pubkey": "11111111111111111111111111111111", "signer": true, "writable": true}
                    ],
                    "instructions": []
                }
            }
        });
        let result = humanize_transaction_to_json(&tx);
        assert!(result.is_object());
        assert!(result.get("signature").is_some());
        assert!(result.get("status").is_some());
        assert!(result.get("fee").is_some());
        assert!(result.get("instructions").is_some());
        assert!(result.get("accounts").is_some());
    }

    #[test]
    fn test_failed_tx_has_success_false() {
        let tx = json!({
            "meta": {
                "err": {"InstructionError": [0, "InsufficientFundsForRent"]},
                "fee": 5000
            },
            "transaction": {
                "signatures": ["FAKESIG123"],
                "message": {
                    "accountKeys": [],
                    "instructions": []
                }
            }
        });
        let result = humanize_transaction_to_json(&tx);
        assert_eq!(result["status"]["success"].as_bool(), Some(false));
    }

    #[test]
    fn test_no_panic_on_null() {
        // Must not panic
        let result = humanize_transaction_to_json(&json!(null));
        // Result is either null or has safe defaults - we just verify no panic
        let _ = result;
    }
}

// =============================================================================
// classify_tx_type tests
// =============================================================================

mod classify_tx_type_tests {
    use serde_json::json;
    use solana_onchain_mcp::tools::tx_inspector::classify_tx_type;

    fn make_tx(instructions: serde_json::Value) -> serde_json::Value {
        json!({ "instructions": instructions })
    }

    #[test]
    fn test_jupiter_instruction_classifies_as_swap() {
        let tx = make_tx(json!([{
            "program": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            "program_name": "Jupiter Aggregator",
            "instruction_type": "JupiterInstruction"
        }]));
        assert_eq!(classify_tx_type(&tx), "swap");
    }

    #[test]
    fn test_raydium_instruction_classifies_as_swap() {
        let tx = make_tx(json!([{
            "program": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
            "program_name": "Raydium DEX",
            "instruction_type": "SwapBaseIn"
        }]));
        assert_eq!(classify_tx_type(&tx), "swap");
    }

    #[test]
    fn test_system_transfer_classifies_as_transfer() {
        let tx = make_tx(json!([{
            "program": "11111111111111111111111111111111",
            "program_name": "System Program",
            "instruction_type": "transfer"
        }]));
        assert_eq!(classify_tx_type(&tx), "transfer");
    }

    #[test]
    fn test_token_mint_to_classifies_as_mint() {
        let tx = make_tx(json!([{
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "program_name": "Token Program",
            "instruction_type": "mintTo"
        }]));
        assert_eq!(classify_tx_type(&tx), "mint");
    }

    #[test]
    fn test_token_burn_classifies_as_burn() {
        let tx = make_tx(json!([{
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "program_name": "Token Program",
            "instruction_type": "burn"
        }]));
        assert_eq!(classify_tx_type(&tx), "burn");
    }

    #[test]
    fn test_unknown_program_instructions_classify_as_unknown() {
        let tx = make_tx(json!([{
            "program": "UnknownProg1111111111111111111111111111111",
            "program_name": null,
            "instruction_type": null
        }]));
        assert_eq!(classify_tx_type(&tx), "unknown");
    }

    #[test]
    fn test_mixed_ata_create_and_swap_classifies_as_swap() {
        // DeFi wins over Core
        let tx = make_tx(json!([
            {
                "program": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
                "program_name": "Associated Token Account",
                "instruction_type": "create"
            },
            {
                "program": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
                "program_name": "Jupiter Aggregator",
                "instruction_type": "JupiterInstruction"
            }
        ]));
        assert_eq!(classify_tx_type(&tx), "swap");
    }

    #[test]
    fn test_empty_instructions_classifies_as_unknown() {
        let tx = make_tx(json!([]));
        assert_eq!(classify_tx_type(&tx), "unknown");
    }

    #[test]
    fn test_no_instructions_key_classifies_as_unknown() {
        let tx = json!({});
        assert_eq!(classify_tx_type(&tx), "unknown");
    }

    // P3-B: NFT branch boundary — no NFT programs are currently in PROGRAM_REGISTRY,
    // so any pubkey that identifies as NFT in reality but is unregistered falls through
    // to "unknown". This test pins the boundary: Metaplex Metadata is the canonical NFT
    // program but is not registered, so it classifies as "unknown".
    #[test]
    fn test_unregistered_nft_program_classifies_as_unknown() {
        let tx = make_tx(json!([{
            "program": "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
            "program_name": "Metaplex Token Metadata",
            "instruction_type": "MintNewEditionFromMasterEditionViaToken"
        }]));
        // Metaplex is not in PROGRAM_REGISTRY → falls through to instruction_type check.
        // "MintNewEditionFromMasterEditionViaToken" contains "mint" but not "mintto",
        // and does not contain "burn" or "transfer", so result is "unknown".
        assert_eq!(classify_tx_type(&tx), "unknown");
    }

    #[test]
    fn test_unknown_program_with_mintto_instruction_type_classifies_as_mint() {
        let tx = make_tx(json!([{
            "program": "SomeProg111111111111111111111111111111111111",
            "instruction_type": "mintTo"
        }]));
        assert_eq!(classify_tx_type(&tx), "mint");
    }
}

// =============================================================================
// format_instruction_error_with_program tests
// =============================================================================

mod format_instruction_error_with_program_tests {
    use serde_json::json;
    use solana_onchain_mcp::tools::tx_inspector::format_instruction_error_with_program;

    const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

    #[test]
    fn custom_error_code_with_known_program_is_interpreted() {
        // Token error code 0 = "Not enough balance"
        let err = json!({ "InstructionError": [0, { "Custom": 0 }] });
        let result = format_instruction_error_with_program(&err, TOKEN_PROGRAM);
        assert!(result.contains("Instruction 0 failed"), "result: {result}");
        assert!(result.contains("Not enough balance"), "result: {result}");
    }

    #[test]
    fn custom_error_code_with_system_program_is_interpreted() {
        // System error code 1 = "Account does not have enough SOL"
        let err = json!({ "InstructionError": [2, { "Custom": 1 }] });
        let result = format_instruction_error_with_program(&err, SYSTEM_PROGRAM);
        assert!(result.contains("Instruction 2 failed"), "result: {result}");
        assert!(result.contains("enough SOL"), "result: {result}");
    }

    #[test]
    fn custom_error_code_with_unknown_program_uses_generic_message() {
        let err = json!({ "InstructionError": [1, { "Custom": 42 }] });
        let result = format_instruction_error_with_program(&err, "UnknownProg1111111111111111111111111111111111");
        assert!(result.contains("Instruction 1 failed"), "result: {result}");
        assert!(result.contains("42"), "result: {result}");
    }

    #[test]
    fn non_custom_error_type_falls_back_to_string_serialization() {
        let err = json!({ "InstructionError": [0, "InsufficientFundsForRent"] });
        let result = format_instruction_error_with_program(&err, TOKEN_PROGRAM);
        assert!(result.contains("Instruction 0 failed"), "result: {result}");
        assert!(
            result.contains("InsufficientFundsForRent"),
            "result: {result}"
        );
    }

    #[test]
    fn non_instruction_error_falls_back_to_json_string() {
        let err = json!({ "OtherError": "Something" });
        let result = format_instruction_error_with_program(&err, TOKEN_PROGRAM);
        assert!(result.contains("OtherError"), "result: {result}");
    }
}
