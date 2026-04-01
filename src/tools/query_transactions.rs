use std::collections::HashMap;

use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};

use crate::{error::SolanaMcpError, rpc::SolanaRpcClient};

use super::{json_result, tx_inspector::{classify_tx_type, humanize_transaction_to_json}};

/// Default number of matched results to return.
const DEFAULT_RESULT_LIMIT: usize = 20;
/// Maximum number of matched results that can be returned.
const MAX_RESULT_LIMIT: usize = 1000;
/// Solana RPC maximum signatures per getSignaturesForAddress call.
const SIGNATURES_PER_PAGE: usize = 1000;
/// v1: scan budget is exactly 1 page.
const MAX_SCAN_PAGES: usize = 1;

#[mcp_tool(
    name = "query_transactions",
    description = "Query and filter transactions for a wallet address with rich classification.

Use this tool when you need to:
- Retrieve a wallet's recent transactions filtered by type (transfer, swap, mint, burn, nft, unknown)
- Summarize transaction activity over a time window (since_days, before/after timestamps)
- Get compact or full humanized transaction details
- Count fees paid and transaction types

Returns a result with matched transactions, type counts, fee totals, and pagination metadata.
Omit 'address' to use the server keypair address."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct QueryTransactionsTool {
    /// Wallet address to query. If omitted, uses the server keypair address.
    pub address: Option<String>,
    /// Number of days back from now to include (mutually exclusive with after_timestamp).
    pub since_days: Option<u64>,
    /// Unix epoch upper bound for block time (exclusive).
    pub before_timestamp: Option<i64>,
    /// Unix epoch lower bound for block time (exclusive). Mutually exclusive with since_days.
    pub after_timestamp: Option<i64>,
    /// Filter by transaction types: "transfer", "swap", "mint", "burn", "nft", "unknown". None = all.
    pub tx_types: Option<Vec<String>>,
    /// Include failed transactions (default false).
    pub include_failed: Option<bool>,
    /// Maximum matched results to return (default 20, max 1000).
    pub limit: Option<u64>,
    /// Return compact output (default true). Set false for full humanized JSON.
    pub compact: Option<bool>,
    /// RPC commitment level: "processed" | "confirmed" | "finalized".
    pub commitment: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompactTx {
    signature: String,
    block_time: Option<i64>,
    #[serde(rename = "type")]
    tx_type: String,
    status: String,
    fee_lamports: u64,
    summary: String,
}

#[derive(Debug, Serialize)]
struct ResultSummary {
    type_counts: HashMap<String, usize>,
    total_fees_lamports: u64,
    failed_count: usize,
}

struct SigEntry {
    signature: String,
    block_time: Option<i64>,
    is_success: bool,
}

#[derive(Debug, Serialize)]
struct QueryTransactionsResult {
    address: String,
    total_scanned: usize,
    matched: usize,
    /// True if the scan exhausted the time window or received fewer than a full page.
    is_complete: bool,
    result_summary: ResultSummary,
    transactions: Vec<serde_json::Value>,
    suggested_actions: Vec<String>,
}

/// Walk `humanized["instructions"]` and return the first non-empty `explanation` string.
/// Falls back to `"Transaction"`.
fn extract_summary(humanized: &serde_json::Value) -> String {
    if let Some(instructions) = humanized.get("instructions").and_then(|i| i.as_array()) {
        for instr in instructions {
            if let Some(explanation) = instr.get("explanation").and_then(|e| e.as_str()) {
                let trimmed = explanation.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    "Transaction".to_string()
}

/// Generate up to 3 deterministic suggested follow-up actions based on the query result.
///
/// Rules applied in order (R-A through R-E); stops as soon as 3 suggestions are collected.
fn generate_suggested_actions(
    results: &[serde_json::Value],
    total_scanned: usize,
    is_complete: bool,
    had_time_filter: bool,
    had_type_filter: bool,
    failed_count: usize,
) -> Vec<String> {
    const MAX_SUGGESTIONS: usize = 3;
    let mut suggestions: Vec<String> = Vec::new();
    let matched = results.len();

    // R-A: zero results with an active time filter
    if matched == 0 && had_time_filter {
        suggestions.push(
            "Try widening the time range — increase since_days or adjust after_timestamp"
                .to_string(),
        );
    }
    if suggestions.len() >= MAX_SUGGESTIONS {
        return suggestions;
    }

    // R-B: scan budget exhausted before time window was fully covered
    if !is_complete {
        suggestions.push(format!(
            "Scan was capped at {total_scanned} signatures. \
             Narrow the time range or add a tx_types filter to reduce volume"
        ));
    }
    if suggestions.len() >= MAX_SUGGESTIONS {
        return suggestions;
    }

    // R-C: for each distinct tx_type in results, suggest inspecting the first example
    let mut type_first_sig: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for tx in results {
        // compact mode has "type" field; full-mode output uses classify_tx_type
        let tx_type = if let Some(t) = tx.get("type").and_then(|v| v.as_str()) {
            t.to_string()
        } else {
            classify_tx_type(tx).to_string()
        };
        let sig = tx
            .get("signature")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        if !sig.is_empty() {
            type_first_sig.entry(tx_type).or_insert(sig);
        }
    }
    // Sort for deterministic output order
    let mut sorted_types: Vec<String> = type_first_sig.keys().cloned().collect();
    sorted_types.sort();
    for tx_type in sorted_types {
        if suggestions.len() >= MAX_SUGGESTIONS {
            return suggestions;
        }
        if let Some(sig) = type_first_sig.get(&tx_type) {
            suggestions.push(format!(
                "Inspect the most recent {tx_type} in detail: \
                 inspect_transaction_humanized with signature {sig}"
            ));
        }
    }
    if suggestions.len() >= MAX_SUGGESTIONS {
        return suggestions;
    }

    // R-D: any failed transactions found in scanned set
    if failed_count > 0 {
        suggestions.push(format!(
            "Found {failed_count} failed transaction(s) in this range. \
             Use inspect_transaction_humanized to diagnose"
        ));
    }
    if suggestions.len() >= MAX_SUGGESTIONS {
        return suggestions;
    }

    // R-E: large result set with no type filter active
    if matched > 20 && !had_type_filter {
        suggestions.push(format!(
            "Large result set ({matched} matches). \
             Add a tx_types filter (e.g. [\"swap\"]) to narrow results"
        ));
    }

    suggestions
}

impl QueryTransactionsTool {
    pub async fn call_tool(
        &self,
        client: &SolanaRpcClient,
        default_address: Option<&str>,
    ) -> Result<CallToolResult, CallToolError> {
        // Resolve address
        let address = match self.address.as_deref().or(default_address) {
            Some(a) => a.to_string(),
            None => {
                return Err(CallToolError::new(SolanaMcpError::RpcError(
                    "No address provided and no keypair configured. Provide 'address' parameter.".to_string(),
                )));
            }
        };

        // Validate mutually exclusive time parameters
        if self.since_days.is_some() && self.after_timestamp.is_some() {
            return Err(CallToolError::new(SolanaMcpError::RpcError(
                "Cannot use both 'since_days' and 'after_timestamp'. Choose one.".to_string(),
            )));
        }

        // Compute after_ts from since_days or after_timestamp
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let after_ts: Option<i64> = if let Some(days) = self.since_days {
            if days > 36_500 {
                return Err(CallToolError::new(SolanaMcpError::RpcError(
                    "since_days exceeds maximum of 36500 (100 years)".to_string(),
                )));
            }
            Some(now_secs - (days as i64) * 86400)
        } else {
            self.after_timestamp
        };

        let before_ts: Option<i64> = self.before_timestamp;

        let effective_limit = (self.limit.unwrap_or(DEFAULT_RESULT_LIMIT as u64) as usize).min(MAX_RESULT_LIMIT);
        let include_failed_flag = self.include_failed.unwrap_or(false);
        let compact_flag = self.compact.unwrap_or(true);

        // Normalise tx_types filter to lowercase for comparison
        let effective_types: Option<Vec<String>> = self
            .tx_types
            .as_ref()
            .map(|types| types.iter().map(|t| t.to_lowercase()).collect());

        // Validate tx_types values against known taxonomy
        const VALID_TX_TYPES: &[&str] = &["transfer", "swap", "mint", "burn", "nft", "unknown"];
        if let Some(types) = &effective_types
            && let Some(bad) = types.iter().find(|t| !VALID_TX_TYPES.contains(&t.as_str()))
        {
            return Err(CallToolError::new(SolanaMcpError::RpcError(format!(
                "Unknown tx_type '{}'. Valid values: transfer, swap, mint, burn, nft, unknown",
                bad
            ))));
        }

        // ---- Phase 1: signature collection ----
        let mut signatures: Vec<SigEntry> = Vec::new();
        let mut cursor: Option<String> = None;
        let mut pages_scanned: usize = 0;
        let mut is_complete = false;

        'pages: while pages_scanned < MAX_SCAN_PAGES {
            let page = client
                .get_signatures_for_address(
                    &address,
                    Some(SIGNATURES_PER_PAGE),
                    cursor.as_deref(),
                    None,
                    self.commitment.as_deref(),
                )
                .await
                .map_err(CallToolError::new)?;

            let entries = match page.as_array() {
                Some(arr) if !arr.is_empty() => arr,
                _ => {
                    is_complete = true;
                    break 'pages;
                }
            };

            for sig_obj in entries {
                let block_time = sig_obj.get("blockTime").and_then(|bt| bt.as_i64());
                let signature = match sig_obj.get("signature").and_then(|s| s.as_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                let err_field = sig_obj.get("err");
                let is_success = err_field.map(|e| e.is_null()).unwrap_or(true);

                // Time gating: skip entries that are too recent
                if let (Some(bt), Some(bts)) = (block_time, before_ts)
                    && bt >= bts
                {
                    continue;
                }

                // Time gating: stop once we pass the lower bound
                if let (Some(bt), Some(ats)) = (block_time, after_ts)
                    && bt < ats
                {
                    is_complete = true;
                    break 'pages;
                }

                signatures.push(SigEntry { signature, block_time, is_success });
            }

            let page_len = entries.len();
            if page_len < SIGNATURES_PER_PAGE {
                is_complete = true;
                break 'pages;
            }

            cursor = signatures.last().map(|e| e.signature.clone());
            pages_scanned += 1;
        }

        let total_scanned = signatures.len();

        // ---- Phase 2: fetch, humanize, filter, project ----
        let mut matched: usize = 0;
        let mut transactions: Vec<serde_json::Value> = Vec::new();
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut total_fees: u64 = 0;
        let mut failed_count: usize = 0;

        for entry in &signatures {
            // Apply include_failed gate before RPC fetch
            if !entry.is_success && !include_failed_flag {
                continue;
            }

            let tx = client
                .get_transaction(&entry.signature, self.commitment.as_deref())
                .await
                .map_err(CallToolError::new)?;

            let humanized = humanize_transaction_to_json(&tx);
            let tx_type = classify_tx_type(&humanized);

            // tx_types filter
            if let Some(type_filter) = &effective_types
                && !type_filter.contains(&tx_type.to_string())
            {
                continue;
            }

            matched += 1;
            *type_counts.entry(tx_type.to_string()).or_insert(0) += 1;

            let fee = humanized.get("fee").and_then(|f| f.as_u64()).unwrap_or(0);
            total_fees = total_fees.saturating_add(fee);

            if !entry.is_success {
                failed_count += 1;
            }

            let tx_value = if compact_flag {
                let summary = extract_summary(&humanized);
                let status = if entry.is_success {
                    "success".to_string()
                } else {
                    "failed".to_string()
                };
                serde_json::to_value(CompactTx {
                    signature: entry.signature.clone(),
                    block_time: entry.block_time,
                    tx_type: tx_type.to_string(),
                    status,
                    fee_lamports: fee,
                    summary,
                })
                .unwrap_or(serde_json::Value::Null)
            } else {
                humanized
            };

            transactions.push(tx_value);

            if matched >= effective_limit {
                break;
            }
        }

        let result_summary = ResultSummary {
            type_counts,
            total_fees_lamports: total_fees,
            failed_count,
        };

        let suggested_actions = generate_suggested_actions(
            &transactions,
            total_scanned,
            is_complete,
            after_ts.is_some() || before_ts.is_some(),
            effective_types.is_some(),
            result_summary.failed_count,
        );

        let result = QueryTransactionsResult {
            address,
            total_scanned,
            matched,
            is_complete,
            result_summary,
            transactions,
            suggested_actions,
        };

        Ok(json_result(&result, "Failed to serialize query_transactions result"))
    }
}

#[cfg(test)]
mod extract_summary_tests {
    use serde_json::json;

    use super::extract_summary;

    #[test]
    fn returns_first_non_empty_explanation() {
        let humanized = json!({
            "instructions": [
                { "explanation": "  " },
                { "explanation": "Transferred 1 SOL to Alice" },
                { "explanation": "Closed account" }
            ]
        });
        assert_eq!(extract_summary(&humanized), "Transferred 1 SOL to Alice");
    }

    #[test]
    fn falls_back_when_all_explanations_empty() {
        let humanized = json!({
            "instructions": [
                { "explanation": "" },
                { "explanation": "   " }
            ]
        });
        assert_eq!(extract_summary(&humanized), "Transaction");
    }

    #[test]
    fn falls_back_when_no_instructions_key() {
        let humanized = json!({ "fee": 5000 });
        assert_eq!(extract_summary(&humanized), "Transaction");
    }

    #[test]
    fn falls_back_on_empty_instructions_array() {
        let humanized = json!({ "instructions": [] });
        assert_eq!(extract_summary(&humanized), "Transaction");
    }
}

#[cfg(test)]
mod suggested_actions_tests {
    use serde_json::json;

    use super::generate_suggested_actions;

    fn compact_tx(sig: &str, tx_type: &str) -> serde_json::Value {
        json!({ "signature": sig, "type": tx_type, "status": "success", "fee_lamports": 5000 })
    }

    #[test]
    fn ra_fires_on_zero_results_with_time_filter() {
        let suggestions = generate_suggested_actions(&[], 0, true, true, false, 0);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("widening the time range"));
    }

    #[test]
    fn ra_does_not_fire_without_time_filter() {
        let suggestions = generate_suggested_actions(&[], 0, true, false, false, 0);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn rb_fires_when_scan_incomplete() {
        let suggestions = generate_suggested_actions(&[], 1000, false, false, false, 0);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("Scan was capped at 1000"));
    }

    #[test]
    fn rc_one_suggestion_per_distinct_type() {
        let txs = vec![
            compact_tx("sig1", "swap"),
            compact_tx("sig2", "transfer"),
            compact_tx("sig3", "mint"),
        ];
        let suggestions = generate_suggested_actions(&txs, 3, true, false, false, 0);
        // R-C: 3 types → 3 suggestions (cap reached)
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions.iter().all(|s| s.contains("inspect_transaction_humanized")));
    }

    #[test]
    fn rc_uses_first_signature_per_type() {
        let txs = vec![
            compact_tx("sig_first_swap", "swap"),
            compact_tx("sig_second_swap", "swap"),
        ];
        let suggestions = generate_suggested_actions(&txs, 2, true, false, false, 0);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("sig_first_swap"));
        assert!(!suggestions[0].contains("sig_second_swap"));
    }

    #[test]
    fn rd_fires_when_failed_count_positive() {
        let txs = vec![compact_tx("sig1", "transfer")];
        let suggestions = generate_suggested_actions(&txs, 1, true, false, false, 2);
        // R-C fires first (1), R-D fires second (2)
        assert!(suggestions.iter().any(|s| s.contains("2 failed transaction")));
    }

    #[test]
    fn re_fires_for_large_result_set_with_no_type_filter() {
        let txs: Vec<serde_json::Value> = (0..25).map(|i| compact_tx(&format!("sig{i}"), "unknown")).collect();
        let suggestions = generate_suggested_actions(&txs, 25, true, false, false, 0);
        // R-C fires for "unknown" (1), R-E fires (2)
        assert!(suggestions.iter().any(|s| s.contains("Large result set")));
    }

    #[test]
    fn re_does_not_fire_when_type_filter_was_set() {
        let txs: Vec<serde_json::Value> = (0..25).map(|i| compact_tx(&format!("sig{i}"), "swap")).collect();
        let suggestions = generate_suggested_actions(&txs, 25, true, false, true, 0);
        assert!(suggestions.iter().all(|s| !s.contains("Large result set")));
    }

    #[test]
    fn max_3_suggestions_regardless_of_rules() {
        // Fire R-A + R-B + R-C (multiple types) + R-D + R-E simultaneously
        let txs: Vec<serde_json::Value> = (0..25)
            .map(|i| {
                let t = ["swap", "transfer", "mint", "burn", "nft"][i % 5];
                compact_tx(&format!("sig{i}"), t)
            })
            .collect();
        // had_time_filter=true (R-A), is_complete=false (R-B), R-C, failed_count=3 (R-D), >20 no filter (R-E)
        let suggestions = generate_suggested_actions(&txs, 1000, false, true, false, 3);
        assert_eq!(suggestions.len(), 3);
    }

    #[test]
    fn empty_results_no_filter_no_failure_returns_empty() {
        let suggestions = generate_suggested_actions(&[], 0, true, false, false, 0);
        assert!(suggestions.is_empty());
    }
}
