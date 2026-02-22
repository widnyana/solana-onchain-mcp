//! Transaction Inspector Programs Registry
//!
//! This module provides a registry of known Solana programs with their
//! instruction parsers and error interpretation.

use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;
use serde_json::json;

use crate::tools::tx_inspector::{ParseError, core::ParsedInstructionData};

/// Program metadata and parsing capabilities
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProgramInfo {
    /// Program public key (base58)
    pub pubkey: String,
    /// Human-readable name
    pub name: String,
    /// Program category
    pub category: ProgramCategory,
    /// Instruction parser function
    pub parser: InstructionParser,
}

/// Categories of Solana programs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum ProgramCategory {
    /// Core Solana programs
    Core,
    /// DeFi protocols (DEXes, lending, etc.)
    DeFi,
    /// NFT and token metadata programs
    NFT,
    /// Bridge programs
    Bridge,
    /// Wallet and authentication programs
    Wallet,
    /// Gaming programs
    Gaming,
    /// Other/uncategorized programs
    Other,
}

/// Function type for parsing instruction data
pub type InstructionParser = fn(&[u8], &[String]) -> Result<ParsedInstructionData, ParseError>;

/// Registry of known programs with their instruction parsers
#[allow(dead_code)]
pub struct ProgramRegistry {
    programs: HashMap<String, ProgramInfo>,
    /// Cache of pubkey -> program name mappings
    name_cache: Mutex<HashMap<String, String>>,
    /// Cache of pubkey -> program category mappings
    category_cache: Mutex<HashMap<String, ProgramCategory>>,
}

#[allow(dead_code)]
impl ProgramRegistry {
    pub fn new() -> Self {
        Self {
            programs: PROGRAM_REGISTRY.iter().map(|p| (p.pubkey.clone(), p.clone())).collect(),
            name_cache: Mutex::new(HashMap::new()),
            category_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Get program info by pubkey
    pub fn get_program(&self, pubkey: &str) -> Option<&ProgramInfo> {
        self.programs.get(pubkey)
    }

    /// Get program name by pubkey
    pub fn get_program_name(&self, pubkey: &str) -> Option<String> {
        {
            let cache = self.name_cache.lock().unwrap();
            if let Some(name) = cache.get(pubkey) {
                return Some(name.clone());
            }
        }

        let name = self.programs.get(pubkey).map(|p| p.name.clone());

        if let Some(ref name) = name {
            let mut cache = self.name_cache.lock().unwrap();
            cache.insert(pubkey.to_string(), name.clone());
        }

        name
    }

    /// Get program category by pubkey
    pub fn get_program_category(&self, pubkey: &str) -> Option<ProgramCategory> {
        {
            let cache = self.category_cache.lock().unwrap();
            if let Some(&category) = cache.get(pubkey) {
                return Some(category);
            }
        }

        let category = self.programs.get(pubkey).map(|p| p.category);

        if let Some(category) = category {
            let mut cache = self.category_cache.lock().unwrap();
            cache.insert(pubkey.to_string(), category);
            Some(category)
        } else {
            None
        }
    }

    /// Check if a pubkey is a known program
    pub fn is_known_program(&self, pubkey: &str) -> bool {
        self.programs.contains_key(pubkey)
    }
}

impl Default for ProgramRegistry {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    /// Static registry of all supported programs
    static ref PROGRAM_REGISTRY: Vec<ProgramInfo> = vec![
        // System Program
        ProgramInfo {
            pubkey: "11111111111111111111111111111111".to_string(),
            name: "System Program".to_string(),
            category: ProgramCategory::Core,
            parser: parse_system_instruction,
        },
        // Token Program
        ProgramInfo {
            pubkey: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            name: "Token Program".to_string(),
            category: ProgramCategory::Core,
            parser: parse_token_instruction,
        },
        // Token-2022 Program
        ProgramInfo {
            pubkey: "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".to_string(),
            name: "Token-2022 Program".to_string(),
            category: ProgramCategory::Core,
            parser: parse_token_instruction,
        },
        // Associated Token Account Program
        ProgramInfo {
            pubkey: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string(),
            name: "Associated Token Account".to_string(),
            category: ProgramCategory::Core,
            parser: parse_ata_instruction,
        },
        // Raydium DEX
        ProgramInfo {
            pubkey: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
            name: "Raydium DEX".to_string(),
            category: ProgramCategory::DeFi,
            parser: parse_raydium_instruction,
        },
        // Jupiter Aggregator
        ProgramInfo {
            pubkey: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
            name: "Jupiter Aggregator".to_string(),
            category: ProgramCategory::DeFi,
            parser: parse_jupiter_instruction,
        },
        // Orca Whirlpool DEX
        ProgramInfo {
            pubkey: "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
            name: "Orca Whirlpool".to_string(),
            category: ProgramCategory::DeFi,
            parser: parse_orca_instruction,
        },
    ];
}

/// System Program instruction types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum SystemInstruction {
    CreateAccount = 0,
    Assign = 1,
    Transfer = 2,
    CreateAccountWithSeed = 3,
    AdvanceNonceAccount = 4,
    WithdrawNonceAccount = 5,
    InitializeNonceAccount = 6,
    AuthorizeNonceAccount = 7,
    Allocate = 8,
    AllocateWithSeed = 9,
    AssignWithSeed = 10,
    TransferWithSeed = 11,
}

impl SystemInstruction {
    fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::CreateAccount),
            1 => Some(Self::Assign),
            2 => Some(Self::Transfer),
            3 => Some(Self::CreateAccountWithSeed),
            4 => Some(Self::AdvanceNonceAccount),
            5 => Some(Self::WithdrawNonceAccount),
            6 => Some(Self::InitializeNonceAccount),
            7 => Some(Self::AuthorizeNonceAccount),
            8 => Some(Self::Allocate),
            9 => Some(Self::AllocateWithSeed),
            10 => Some(Self::AssignWithSeed),
            11 => Some(Self::TransferWithSeed),
            _ => None,
        }
    }
}

/// Parse System Program instruction
fn parse_system_instruction(data: &[u8], _accounts: &[String]) -> Result<ParsedInstructionData, ParseError> {
    if data.is_empty() {
        return Ok(ParsedInstructionData::Raw(
            "Empty instruction data".to_string(),
        ));
    }

    let instruction_type = SystemInstruction::from_u8(data[0]);

    match instruction_type {
        Some(SystemInstruction::CreateAccount) => {
            if data.len() >= 4 + 32 + 8 {
                let lamports = u32::from_le_bytes(data[1..5].try_into().unwrap()) as u64;
                let space = u64::from_le_bytes(data[5..13].try_into().unwrap());
                let owner = solana_sdk::bs58::encode(&data[13..45]).into_string();

                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "CreateAccount",
                    "lamports": lamports,
                    "space": space,
                    "owner": owner,
                })))
            } else {
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "CreateAccount",
                    "note": "Incomplete data"
                })))
            }
        }
        Some(SystemInstruction::Assign) => {
            if data.len() > 32 {
                let owner = solana_sdk::bs58::encode(&data[1..33]).into_string();
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "Assign",
                    "owner": owner,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "Assign (incomplete data)".to_string(),
                ))
            }
        }
        Some(SystemInstruction::Transfer) => {
            if data.len() > 8 {
                let lamports = u64::from_le_bytes(data[1..9].try_into().unwrap());
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "Transfer",
                    "lamports": lamports,
                    "sol": format!("{} SOL", lamports as f64 / 1_000_000_000.0),
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "Transfer (incomplete data)".to_string(),
                ))
            }
        }
        Some(SystemInstruction::CreateAccountWithSeed) => {
            let seed_start = 1 + 32;
            let seed_len_pos = seed_start;

            if data.len() >= seed_len_pos + 4 {
                let seed_len = u32::from_le_bytes(data[seed_len_pos..seed_len_pos + 4].try_into().unwrap()) as usize;
                let seed_end = seed_len_pos + 4 + seed_len;

                if data.len() >= seed_end + 32 + 8 {
                    let seed = String::from_utf8_lossy(&data[seed_len_pos + 4..seed_end]).to_string();
                    let lamports = u64::from_le_bytes(data[seed_end..seed_end + 8].try_into().unwrap());
                    let space = u64::from_le_bytes(data[seed_end + 8..seed_end + 16].try_into().unwrap());
                    let owner = solana_sdk::bs58::encode(&data[seed_end + 16..seed_end + 48]).into_string();

                    Ok(ParsedInstructionData::Decoded(json!({
                        "type": "CreateAccountWithSeed",
                        "seed": seed,
                        "lamports": lamports,
                        "space": space,
                        "owner": owner,
                    })))
                } else {
                    Ok(ParsedInstructionData::Raw(
                        "CreateAccountWithSeed (incomplete data)".to_string(),
                    ))
                }
            } else {
                Ok(ParsedInstructionData::Raw(
                    "CreateAccountWithSeed (incomplete data)".to_string(),
                ))
            }
        }
        _ => Ok(ParsedInstructionData::Raw(format!(
            "Unknown System instruction: {}",
            data[0]
        ))),
    }
}

/// Token Program instruction types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum TokenInstruction {
    InitializeMint = 0,
    InitializeAccount = 1,
    InitializeMultisig = 2,
    Transfer = 3,
    Approve = 4,
    Revoke = 5,
    SetAuthority = 6,
    MintTo = 7,
    Burn = 8,
    CloseAccount = 9,
    FreezeAccount = 10,
    ThawAccount = 11,
    Transfer2 = 12,
    Approve2 = 13,
    Revoke2 = 14,
    FreezeAccount2 = 15,
    ThawAccount2 = 16,
    MintTo2 = 17,
    Burn2 = 18,
    InitializeAccount2 = 19,
    SyncNative = 20,
    InitializeAccount3 = 21,
    InitializeMultisig2 = 22,
    InitializeMint2 = 23,
    GetAccountDataSize = 24,
    InitializeImmutableOwner = 25,
    AmountToUiAmount = 26,
    UiAmountToAmount = 27,
    InitializeMintCloseAuthority = 28,
    TransferFeeExtension = 29,
    ConfidentialTransferExtension = 30,
    DefaultAccountStateExtension = 31,
    Reallocate = 32,
    MemoTransferExtension = 33,
    CreateNativeMint = 34,
    InitializeNonTransferableMint = 35,
    InterestBearingMintExtension = 36,
}

impl TokenInstruction {
    fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::InitializeMint),
            1 => Some(Self::InitializeAccount),
            2 => Some(Self::InitializeMultisig),
            3 => Some(Self::Transfer),
            4 => Some(Self::Approve),
            5 => Some(Self::Revoke),
            6 => Some(Self::SetAuthority),
            7 => Some(Self::MintTo),
            8 => Some(Self::Burn),
            9 => Some(Self::CloseAccount),
            10 => Some(Self::FreezeAccount),
            11 => Some(Self::ThawAccount),
            12 => Some(Self::Transfer2),
            13 => Some(Self::Approve2),
            14 => Some(Self::Revoke2),
            15 => Some(Self::FreezeAccount2),
            16 => Some(Self::ThawAccount2),
            17 => Some(Self::MintTo2),
            18 => Some(Self::Burn2),
            19 => Some(Self::InitializeAccount2),
            20 => Some(Self::SyncNative),
            21 => Some(Self::InitializeAccount3),
            22 => Some(Self::InitializeMultisig2),
            23 => Some(Self::InitializeMint2),
            24 => Some(Self::GetAccountDataSize),
            25 => Some(Self::InitializeImmutableOwner),
            26 => Some(Self::AmountToUiAmount),
            27 => Some(Self::UiAmountToAmount),
            28 => Some(Self::InitializeMintCloseAuthority),
            29 => Some(Self::TransferFeeExtension),
            30 => Some(Self::ConfidentialTransferExtension),
            31 => Some(Self::DefaultAccountStateExtension),
            32 => Some(Self::Reallocate),
            33 => Some(Self::MemoTransferExtension),
            34 => Some(Self::CreateNativeMint),
            35 => Some(Self::InitializeNonTransferableMint),
            36 => Some(Self::InterestBearingMintExtension),
            _ => None,
        }
    }
}

/// Parse Token Program instruction
fn parse_token_instruction(data: &[u8], _accounts: &[String]) -> Result<ParsedInstructionData, ParseError> {
    if data.is_empty() {
        return Ok(ParsedInstructionData::Raw(
            "Empty instruction data".to_string(),
        ));
    }

    let instruction_type = TokenInstruction::from_u8(data[0]);

    match instruction_type {
        Some(TokenInstruction::InitializeMint) => {
            if data.len() > 9 {
                let decimals = data[1];
                let mint_authority_option = data[2];
                let freeze_authority_option = if data.len() > 10 { data[10] } else { 0 };

                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "InitializeMint",
                    "decimals": decimals,
                    "mint_authority": mint_authority_option == 1,
                    "freeze_authority": freeze_authority_option == 1,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "InitializeMint (incomplete data)".to_string(),
                ))
            }
        }
        Some(TokenInstruction::InitializeAccount) | Some(TokenInstruction::InitializeAccount2) => {
            Ok(ParsedInstructionData::Decoded(json!({
                "type": "InitializeAccount",
            })))
        }
        Some(TokenInstruction::Transfer) | Some(TokenInstruction::Transfer2) => {
            if data.len() > 8 {
                let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "Transfer",
                    "amount": amount,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "Transfer (incomplete data)".to_string(),
                ))
            }
        }
        Some(TokenInstruction::MintTo) | Some(TokenInstruction::MintTo2) => {
            if data.len() > 8 {
                let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "MintTo",
                    "amount": amount,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "MintTo (incomplete data)".to_string(),
                ))
            }
        }
        Some(TokenInstruction::Burn) | Some(TokenInstruction::Burn2) => {
            if data.len() > 8 {
                let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "Burn",
                    "amount": amount,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "Burn (incomplete data)".to_string(),
                ))
            }
        }
        Some(TokenInstruction::CloseAccount) => Ok(ParsedInstructionData::Decoded(json!({
            "type": "CloseAccount",
        }))),
        Some(TokenInstruction::Approve) | Some(TokenInstruction::Approve2) => {
            if data.len() > 8 {
                let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "Approve",
                    "amount": amount,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "Approve (incomplete data)".to_string(),
                ))
            }
        }
        Some(TokenInstruction::SetAuthority) => {
            if data.len() >= 2 {
                let authority_type = data[1];
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "SetAuthority",
                    "authority_type": authority_type,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "SetAuthority (incomplete data)".to_string(),
                ))
            }
        }
        Some(TokenInstruction::FreezeAccount) | Some(TokenInstruction::FreezeAccount2) => {
            Ok(ParsedInstructionData::Decoded(json!({
                "type": "FreezeAccount",
            })))
        }
        Some(TokenInstruction::ThawAccount) | Some(TokenInstruction::ThawAccount2) => {
            Ok(ParsedInstructionData::Decoded(json!({
                "type": "ThawAccount",
            })))
        }
        Some(TokenInstruction::Revoke) | Some(TokenInstruction::Revoke2) => Ok(ParsedInstructionData::Decoded(json!({
            "type": "Revoke",
        }))),
        Some(TokenInstruction::InitializeMultisig) | Some(TokenInstruction::InitializeMultisig2) => {
            if data.len() >= 2 {
                let m = data[1];
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "InitializeMultisig",
                    "signers_required": m,
                })))
            } else {
                Ok(ParsedInstructionData::Raw(
                    "InitializeMultisig (incomplete data)".to_string(),
                ))
            }
        }
        Some(TokenInstruction::SyncNative) => Ok(ParsedInstructionData::Decoded(json!({
            "type": "SyncNative",
        }))),
        _ => Ok(ParsedInstructionData::Raw(format!(
            "Unknown Token instruction: {}",
            data[0]
        ))),
    }
}

/// Associated Token Account instruction types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AtaInstruction {
    Create = 0,
    CreateIdempotent = 1,
    RecoverNested = 2,
}

impl AtaInstruction {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Create),
            1 => Some(Self::CreateIdempotent),
            2 => Some(Self::RecoverNested),
            _ => None,
        }
    }
}

/// Parse Associated Token Account instruction
fn parse_ata_instruction(data: &[u8], _accounts: &[String]) -> Result<ParsedInstructionData, ParseError> {
    if data.is_empty() {
        return Ok(ParsedInstructionData::Raw(
            "Empty instruction data".to_string(),
        ));
    }

    let instruction_type = AtaInstruction::from_u8(data[0]);

    match instruction_type {
        Some(AtaInstruction::Create) => Ok(ParsedInstructionData::Decoded(json!({
            "type": "CreateAssociatedTokenAccount",
        }))),
        Some(AtaInstruction::CreateIdempotent) => Ok(ParsedInstructionData::Decoded(json!({
            "type": "CreateAssociatedTokenAccountIdempotent",
        }))),
        Some(AtaInstruction::RecoverNested) => Ok(ParsedInstructionData::Decoded(json!({
            "type": "RecoverNested",
        }))),
        _ => Ok(ParsedInstructionData::Raw(format!(
            "Unknown ATA instruction: {}",
            data[0]
        ))),
    }
}

/// Raydium AMM discriminators (Anchor: sha256("global:<instruction_name>")[0..8])
mod raydium {
    pub const SWAP_BASE_IN: [u8; 8] = [0x2a, 0xec, 0x48, 0xa2, 0xf2, 0x18, 0x27, 0x54];
    pub const SWAP_BASE_OUT: [u8; 8] = [0xa3, 0xd2, 0x9b, 0xd0, 0xaf, 0x92, 0xd5, 0x96];
    pub const ADD_LIQUIDITY: [u8; 8] = [0xb5, 0x9d, 0x59, 0x43, 0x8f, 0xb6, 0x34, 0x48];
    pub const REMOVE_LIQUIDITY: [u8; 8] = [0x50, 0x55, 0xd1, 0x48, 0x18, 0xce, 0xb1, 0x6c];
}

/// Parse Raydium AMM instruction
fn parse_raydium_instruction(data: &[u8], accounts: &[String]) -> Result<ParsedInstructionData, ParseError> {
    if data.len() < 8 {
        return Ok(ParsedInstructionData::Raw(
            "Raydium instruction data too short".to_string(),
        ));
    }

    let discriminator = &data[0..8];

    match discriminator {
        d if *d == raydium::SWAP_BASE_IN => {
            // swap_base_in: [discriminator:8][amount_in:8][minimum_amount_out:8]
            if data.len() >= 24 {
                let amount_in = u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8]));
                let min_out = u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8]));

                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "SwapBaseIn",
                    "description": "Swap tokens with fixed input amount",
                    "amount_in": amount_in,
                    "minimum_amount_out": min_out,
                    "source_account": accounts.first().unwrap_or(&"unknown".to_string()),
                    "destination_account": accounts.get(1).unwrap_or(&"unknown".to_string()),
                })))
            } else {
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "SwapBaseIn",
                    "note": "Incomplete data"
                })))
            }
        }
        d if *d == raydium::SWAP_BASE_OUT => {
            // swap_base_out: [discriminator:8][max_amount_in:8][amount_out:8]
            if data.len() >= 24 {
                let max_in = u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8]));
                let amount_out = u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8]));

                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "SwapBaseOut",
                    "description": "Swap tokens with fixed output amount",
                    "max_amount_in": max_in,
                    "amount_out": amount_out,
                    "source_account": accounts.first().unwrap_or(&"unknown".to_string()),
                    "destination_account": accounts.get(1).unwrap_or(&"unknown".to_string()),
                })))
            } else {
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "SwapBaseOut",
                    "note": "Incomplete data"
                })))
            }
        }
        d if *d == raydium::ADD_LIQUIDITY => Ok(ParsedInstructionData::Decoded(json!({
            "type": "AddLiquidity",
            "description": "Add liquidity to a pool",
        }))),
        d if *d == raydium::REMOVE_LIQUIDITY => Ok(ParsedInstructionData::Decoded(json!({
            "type": "RemoveLiquidity",
            "description": "Remove liquidity from a pool",
        }))),
        _ => {
            let disc_hex = hex::encode(discriminator);
            Ok(ParsedInstructionData::Decoded(json!({
                "type": "UnknownRaydiumInstruction",
                "discriminator": disc_hex,
            })))
        }
    }
}

/// Jupiter instruction parsing (placeholder for future expansion)
fn parse_jupiter_instruction(data: &[u8], _accounts: &[String]) -> Result<ParsedInstructionData, ParseError> {
    if !data.is_empty() {
        Ok(ParsedInstructionData::Decoded(json!({
            "type": "JupiterInstruction",
            "instruction": data[0],
            "note": "Full instruction parsing not yet implemented"
        })))
    } else {
        Ok(ParsedInstructionData::Raw(
            "Jupiter instruction data".to_string(),
        ))
    }
}

/// Orca Whirlpool discriminators (Anchor: sha256("global:<instruction_name>")[0..8])
mod orca_whirlpool {
    pub const SWAP: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8];
    pub const INCREASE_LIQUIDITY: [u8; 8] = [0x2e, 0x9c, 0xf3, 0x76, 0x0d, 0xcd, 0xfb, 0xb2];
    pub const DECREASE_LIQUIDITY: [u8; 8] = [0xa0, 0x26, 0xd0, 0x6f, 0x68, 0x5b, 0x2c, 0x01];
    pub const INITIALIZE_POOL: [u8; 8] = [0x5f, 0xb4, 0x0a, 0xac, 0x54, 0xae, 0xe8, 0x28];
}

/// Parse Orca Whirlpool instruction
fn parse_orca_instruction(data: &[u8], accounts: &[String]) -> Result<ParsedInstructionData, ParseError> {
    if data.len() < 8 {
        return Ok(ParsedInstructionData::Raw(
            "Orca Whirlpool instruction data too short".to_string(),
        ));
    }

    let discriminator = &data[0..8];

    match discriminator {
        d if *d == orca_whirlpool::SWAP => {
            // Whirlpool swap: [discriminator:8][amount:8][other_threshold:8][sqrt_price_limit:16][amount_specified_is_input:1][a_to_b:1]
            if data.len() >= 42 {
                let amount = u64::from_le_bytes(data[8..16].try_into().unwrap_or([0; 8]));
                let other_threshold = u64::from_le_bytes(data[16..24].try_into().unwrap_or([0; 8]));
                let amount_specified_is_input = data.get(40).copied().unwrap_or(0) == 1;
                let a_to_b = data.get(41).copied().unwrap_or(0) == 1;

                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "Swap",
                    "description": "Swap tokens in Whirlpool",
                    "amount": amount,
                    "other_amount_threshold": other_threshold,
                    "amount_specified_is_input": amount_specified_is_input,
                    "a_to_b": a_to_b,
                    "whirlpool": accounts.first().unwrap_or(&"unknown".to_string()),
                    "token_owner_account_a": accounts.get(1).unwrap_or(&"unknown".to_string()),
                    "token_owner_account_b": accounts.get(2).unwrap_or(&"unknown".to_string()),
                })))
            } else {
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "Swap",
                    "description": "Swap tokens in Whirlpool",
                    "note": "Incomplete data"
                })))
            }
        }
        d if *d == orca_whirlpool::INCREASE_LIQUIDITY => {
            // increase_liquidity: [discriminator:8][liquidity_amount:16][token_max_a:8][token_max_b:8]
            if data.len() >= 40 {
                let liquidity_amount = u128::from_le_bytes({
                    let mut arr = [0u8; 16];
                    arr.copy_from_slice(&data[8..24]);
                    arr
                });
                let token_max_a = u64::from_le_bytes(data[24..32].try_into().unwrap_or([0; 8]));
                let token_max_b = u64::from_le_bytes(data[32..40].try_into().unwrap_or([0; 8]));

                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "IncreaseLiquidity",
                    "description": "Add liquidity to a Whirlpool position",
                    "liquidity_amount": liquidity_amount.to_string(),
                    "token_max_a": token_max_a,
                    "token_max_b": token_max_b,
                })))
            } else {
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "IncreaseLiquidity",
                    "description": "Add liquidity to a Whirlpool position",
                    "note": "Incomplete data"
                })))
            }
        }
        d if *d == orca_whirlpool::DECREASE_LIQUIDITY => {
            // decrease_liquidity: [discriminator:8][liquidity_amount:16][token_min_a:8][token_min_b:8]
            if data.len() >= 40 {
                let liquidity_amount = u128::from_le_bytes({
                    let mut arr = [0u8; 16];
                    arr.copy_from_slice(&data[8..24]);
                    arr
                });
                let token_min_a = u64::from_le_bytes(data[24..32].try_into().unwrap_or([0; 8]));
                let token_min_b = u64::from_le_bytes(data[32..40].try_into().unwrap_or([0; 8]));

                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "DecreaseLiquidity",
                    "description": "Remove liquidity from a Whirlpool position",
                    "liquidity_amount": liquidity_amount.to_string(),
                    "token_min_a": token_min_a,
                    "token_min_b": token_min_b,
                })))
            } else {
                Ok(ParsedInstructionData::Decoded(json!({
                    "type": "DecreaseLiquidity",
                    "description": "Remove liquidity from a Whirlpool position",
                    "note": "Incomplete data"
                })))
            }
        }
        d if *d == orca_whirlpool::INITIALIZE_POOL => Ok(ParsedInstructionData::Decoded(json!({
            "type": "InitializePool",
            "description": "Initialize a new Whirlpool",
        }))),
        _ => {
            let disc_hex = hex::encode(discriminator);
            Ok(ParsedInstructionData::Decoded(json!({
                "type": "UnknownOrcaInstruction",
                "discriminator": disc_hex,
            })))
        }
    }
}

/// System Program error codes
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
#[allow(dead_code)]
pub enum SystemError {
    /// An account with the same address already exists
    AccountAlreadyInUse = 0,
    /// Account does not have enough SOL for the requested operation
    ResultWithNegativeLamports = 1,
    /// Cannot assign account to this program
    InvalidAccountOwner = 2,
    /// Cannot allocate account data for this program
    InvalidAccountData,
    /// Account data too small for specified instruction
    AccountDataTooSmall,
    /// Insufficient funds for operation
    InsufficientFunds,
    /// Invalid program ID for this instruction
    InvalidProgramId,
    /// Invalid owner for this operation
    InvalidOwner,
    /// Account already exists
    AccountAlreadyExists,
    /// Account does not exist
    AccountNotFound,
    /// Account is not writable
    AccountNotSufficient,
    /// External account chain has too many accounts
    ExternalAccountChainTooDeep,
    /// Invalid account index
    InvalidAccountIndex,
    /// Invalid length encoding
    InvalidLength,
    /// Account support for loading multiple program
    MaxSeedLengthExceeded,
    /// Invalid seeds for address generation
    InvalidSeeds,
    /// Non-writable account cannot be changed
    AccountNotWritable,
}

/// Token Program error codes
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
#[allow(dead_code)]
pub enum TokenError {
    /// Not enough balance for the requested operation
    NotEnoughBalance = 0,
    /// Invalid owner for this token account
    InvalidOwner,
    /// Account is frozen
    AccountFrozen,
    /// Invalid mint
    InvalidMint,
    /// Token account not initialized
    AccountNotInitialized,
    /// Mint not initialized
    MintNotInitialized,
    /// Invalid state for account
    InvalidState,
    /// Invalid instruction
    InvalidInstruction,
    /// Mint mismatch
    MintMismatch,
    /// Account position mismatch
    AccountPositionMismatch,
    /// Required signature for instruction missing
    MissingSignature,
    /// Authority does not match
    AuthorityTypeNotSupported,
    /// Token amount overflow
    Overflow,
    /// Authority type does not match account
    AuthorityTypeNotMatchAccount,
    /// Invalid account length
    InvalidAccountLength,
    /// Account is not a mint
    AccountNotMint,
    /// Account is not a token account
    AccountNotToken,
    /// Duplicate account index
    DuplicateAccountIndex,
    /// Owner mismatch
    OwnerMismatch,
    /// Invalid owner
    InvalidOwnerForOperation,
    /// Authority type is not valid
    InvalidAuthorityType,
    /// Freeze authority cannot be set
    NoFreezeAuthority,
    /// Mint cannot be frozen
    MintCannotFreeze,
    /// Account is already frozen
    AccountAlreadyFrozen,
    /// Mint decimals mismatch
    MintDecimalsMismatch,
    /// Non-writable account cannot be modified
    AccountNotWritable,
    /// Extension not available for Token program
    ExtensionNotAvailable,
    /// Extension is required for this instruction
    ExtensionIsRequired,
    /// Invalid extension for operation
    InvalidExtensionForOperation,
    /// Conflicting extensions
    ConflictingExtensions,
}

/// Interpret a program error code into a human-readable message
pub fn interpret_error(program_id: &str, error_code: u32) -> String {
    if program_id == "11111111111111111111111111111111" {
        match error_code {
            0 => "An account with the same address already exists".to_string(),
            1 => "Account does not have enough SOL for the requested operation".to_string(),
            2 => "Cannot assign account to this program".to_string(),
            3 => "Cannot allocate account data for this program".to_string(),
            4 => "Account data too small for specified instruction".to_string(),
            5 => "Insufficient funds for operation".to_string(),
            6 => "Invalid program ID for this instruction".to_string(),
            7 => "Invalid owner for this operation".to_string(),
            8 => "Account already exists".to_string(),
            9 => "Account does not exist".to_string(),
            10 => "Account is not writable".to_string(),
            11 => "External account chain has too many accounts".to_string(),
            12 => "Invalid account index".to_string(),
            13 => "Invalid length encoding".to_string(),
            14 => "Max seed length exceeded for address generation".to_string(),
            15 => "Invalid seeds for address generation".to_string(),
            16 => "Non-writable account cannot be changed".to_string(),
            _ => format!("Unknown System error code: {}", error_code),
        }
    } else if program_id == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        || program_id == "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
    {
        match error_code {
            0 => "Not enough balance for the requested operation".to_string(),
            1 => "Invalid owner for this token account".to_string(),
            2 => "Account is frozen".to_string(),
            3 => "Invalid mint".to_string(),
            4 => "Token account not initialized".to_string(),
            5 => "Mint not initialized".to_string(),
            6 => "Invalid state for account".to_string(),
            7 => "Invalid instruction".to_string(),
            8 => "Mint mismatch".to_string(),
            9 => "Account position mismatch".to_string(),
            10 => "Required signature for instruction missing".to_string(),
            11 => "Authority type not supported".to_string(),
            12 => "Token amount overflow".to_string(),
            13 => "Authority type does not match account".to_string(),
            14 => "Invalid account length".to_string(),
            15 => "Account is not a mint".to_string(),
            16 => "Account is not a token account".to_string(),
            17 => "Duplicate account index".to_string(),
            18 => "Owner mismatch".to_string(),
            19 => "Invalid owner for operation".to_string(),
            20 => "Invalid authority type".to_string(),
            21 => "No freeze authority".to_string(),
            22 => "Mint cannot freeze".to_string(),
            23 => "Account is already frozen".to_string(),
            24 => "Mint decimals mismatch".to_string(),
            25 => "Account is not writable".to_string(),
            26 => "Extension not available for Token program".to_string(),
            27 => "Extension is required for this instruction".to_string(),
            28 => "Invalid extension for operation".to_string(),
            29 => "Conflicting extensions".to_string(),
            _ => format!("Unknown Token error code: {}", error_code),
        }
    } else if program_id == "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" {
        // Raydium AMM errors (top 10 most common)
        match error_code {
            0 => "Invalid pool state".to_string(),
            1 => "Insufficient input amount".to_string(),
            2 => "Slippage tolerance exceeded".to_string(),
            3 => "Insufficient output amount".to_string(),
            4 => "Invalid token program".to_string(),
            5 => "Pool token mint mismatch".to_string(),
            6 => "Invalid fee account".to_string(),
            7 => "Math overflow".to_string(),
            8 => "Invalid authority".to_string(),
            9 => "Invalid account owner".to_string(),
            _ => format!("Unknown Raydium error code: {}", error_code),
        }
    } else if program_id == "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" {
        // Orca Whirlpool errors (top 10 most common)
        match error_code {
            0 => "Invalid whirlpool".to_string(),
            1 => "Invalid tick array".to_string(),
            2 => "Invalid position".to_string(),
            3 => "Invalid token mint".to_string(),
            4 => "Sqrt price out of range".to_string(),
            5 => "Liquidity zero".to_string(),
            6 => "Tick array sequence invalid".to_string(),
            7 => "Invalid tick index".to_string(),
            8 => "Fee rate exceeds max".to_string(),
            9 => "Protocol fee rate exceeds max".to_string(),
            _ => format!("Unknown Orca Whirlpool error code: {}", error_code),
        }
    } else {
        format!("Error code {} for program {}", error_code, program_id)
    }
}

/// Identify a program by its public key
pub fn identify_program(program_id: &str) -> Option<&'static ProgramInfo> {
    PROGRAM_REGISTRY.iter().find(|p| p.pubkey == program_id)
}

/// Decode instruction data using the appropriate parser
pub fn decode_instruction(
    program_id: &str,
    data: &[u8],
    accounts: &[String],
) -> Result<ParsedInstructionData, ParseError> {
    if let Some(program) = identify_program(program_id) {
        (program.parser)(data, accounts)
    } else {
        Ok(ParsedInstructionData::Raw(hex::encode(data)))
    }
}
