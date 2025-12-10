//! Types for the Polymarket Relayer Client
//!
//! This module contains all types used for interacting with Polymarket's
//! Polygon relayer infrastructure for gasless transactions.

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a number or string to String
fn deserialize_number_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Float(f64),
        Int(i64),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => Ok(s),
        StringOrNumber::Float(f) => Ok(f.to_string()),
        StringOrNumber::Int(i) => Ok(i.to_string()),
    }
}

/// Builder API credentials for relayer authentication
///
/// **Important**: These credentials are different from CLOB API credentials!
///
/// To obtain Builder API credentials:
/// 1. Go to https://polymarket.com/settings?tab=builder
/// 2. Click "+ Create New" in the Builder Keys section
/// 3. Save your apiKey, secret, and passphrase
///
/// Environment variables:
/// - `POLY_API_KEY`: The API key (apiKey from Builder Keys)
/// - `POLY_API_SECRET`: The API secret (secret from Builder Keys)
/// - `POLY_PASSPHRASE`: The API passphrase (passphrase from Builder Keys)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderApiCreds {
    pub key: String,
    pub secret: String,
    pub passphrase: String,
}

impl BuilderApiCreds {
    pub fn new(key: String, secret: String, passphrase: String) -> Self {
        Self {
            key,
            secret,
            passphrase,
        }
    }

    /// Create BuilderApiCreds from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `POLY_API_KEY`: The API key
    /// - `POLY_API_SECRET`: The API secret
    /// - `POLY_PASSPHRASE`: The API passphrase
    ///
    /// Returns `None` if any of the required environment variables are not set.
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("POLY_API_KEY").ok()?;
        let secret = std::env::var("POLY_API_SECRET").ok()?;
        let passphrase = std::env::var("POLY_PASSPHRASE").ok()?;

        Some(Self {
            key,
            secret,
            passphrase,
        })
    }
}

/// Operation type for Safe transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OperationType {
    Call = 0,
    DelegateCall = 1,
}

impl From<OperationType> for u8 {
    fn from(op: OperationType) -> Self {
        op as u8
    }
}

impl Default for OperationType {
    fn default() -> Self {
        OperationType::Call
    }
}

/// Transaction type for relayer requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Safe,
    SafeCreate,
    Proxy,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Safe => "SAFE",
            TransactionType::SafeCreate => "SAFE-CREATE",
            TransactionType::Proxy => "PROXY",
        }
    }
}

/// A single Safe transaction
#[derive(Debug, Clone)]
pub struct SafeTransaction {
    pub to: String,
    pub operation: OperationType,
    pub data: String,
    pub value: String,
}

impl SafeTransaction {
    /// Create a new Safe transaction
    pub fn new(to: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            operation: OperationType::Call,
            data: data.into(),
            value: "0".to_string(),
        }
    }

    /// Set the operation type
    pub fn operation(mut self, operation: OperationType) -> Self {
        self.operation = operation;
        self
    }

    /// Set the value (in wei)
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
}

/// Arguments for building a Safe transaction request
#[derive(Debug, Clone)]
pub struct SafeTransactionArgs {
    pub from_address: String,
    pub nonce: String,
    pub chain_id: u64,
    pub transactions: Vec<SafeTransaction>,
}

/// Signature parameters for Safe transactions
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_txn_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_receiver: Option<String>,
    // For SAFE-CREATE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_receiver: Option<String>,
}

impl Default for SignatureParams {
    fn default() -> Self {
        Self {
            gas_price: None,
            operation: None,
            safe_txn_gas: None,
            base_gas: None,
            gas_token: None,
            refund_receiver: None,
            payment_token: None,
            payment: None,
            payment_receiver: None,
        }
    }
}

impl SignatureParams {
    /// Create signature params for Safe transaction execution
    pub fn for_safe_execution(operation: OperationType) -> Self {
        Self {
            gas_price: Some("0".to_string()),
            operation: Some((operation as u8).to_string()),
            safe_txn_gas: Some("0".to_string()),
            base_gas: Some("0".to_string()),
            gas_token: Some(ZERO_ADDRESS.to_string()),
            refund_receiver: Some(ZERO_ADDRESS.to_string()),
            payment_token: None,
            payment: None,
            payment_receiver: None,
        }
    }

    /// Create signature params for Safe creation
    pub fn for_safe_create() -> Self {
        Self {
            gas_price: None,
            operation: None,
            safe_txn_gas: None,
            base_gas: None,
            gas_token: None,
            refund_receiver: None,
            payment_token: Some(ZERO_ADDRESS.to_string()),
            payment: Some("0".to_string()),
            payment_receiver: Some(ZERO_ADDRESS.to_string()),
        }
    }
}

/// Transaction request to submit to the relayer
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub from: String,
    pub to: String,
    pub proxy_wallet: String,
    pub data: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_params: Option<SignatureParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

/// State of a relayer transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayerTransactionState {
    #[serde(rename = "STATE_NEW")]
    New,
    #[serde(rename = "STATE_EXECUTED")]
    Executed,
    #[serde(rename = "STATE_MINED")]
    Mined,
    #[serde(rename = "STATE_CONFIRMED")]
    Confirmed,
    #[serde(rename = "STATE_FAILED")]
    Failed,
    #[serde(rename = "STATE_INVALID")]
    Invalid,
}

impl RelayerTransactionState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            RelayerTransactionState::Confirmed
                | RelayerTransactionState::Failed
                | RelayerTransactionState::Invalid
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(
            self,
            RelayerTransactionState::Mined | RelayerTransactionState::Confirmed
        )
    }
}

/// Response from submitting a transaction to the relayer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerSubmitResponse {
    #[serde(rename = "transactionID")]
    pub transaction_id: String,
    #[serde(rename = "transactionHash", default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
}

/// Full relayer transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerTransaction {
    #[serde(rename = "transactionID")]
    pub transaction_id: String,
    #[serde(rename = "transactionHash", default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(rename = "proxyAddress", default)]
    pub proxy_address: Option<String>,
    #[serde(default)]
    pub data: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(rename = "type", default)]
    pub tx_type: Option<String>,
    #[serde(default)]
    pub metadata: Option<String>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
}

impl RelayerTransaction {
    pub fn get_state(&self) -> Option<RelayerTransactionState> {
        self.state.as_ref().and_then(|s| match s.as_str() {
            "STATE_NEW" => Some(RelayerTransactionState::New),
            "STATE_EXECUTED" => Some(RelayerTransactionState::Executed),
            "STATE_MINED" => Some(RelayerTransactionState::Mined),
            "STATE_CONFIRMED" => Some(RelayerTransactionState::Confirmed),
            "STATE_FAILED" => Some(RelayerTransactionState::Failed),
            "STATE_INVALID" => Some(RelayerTransactionState::Invalid),
            _ => None,
        })
    }
}

/// Response from nonce endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
}

/// Response from deployed endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedResponse {
    pub deployed: bool,
}

/// Relayer contract configuration
#[derive(Debug, Clone)]
pub struct RelayerContractConfig {
    pub safe_factory: String,
    pub safe_multisend: String,
    pub ctf: String,
    pub collateral: String,
}

/// Constants
pub const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub const SAFE_INIT_CODE_HASH: &str =
    "0x2bce2127ff07fb632d16c8347c4ebf501f4841168bed00d9e6ef715ddb6fcecf";
pub const SAFE_FACTORY_NAME: &str = "Polymarket Contract Proxy Factory";

/// Polygon Mainnet relayer configuration
pub fn mainnet_relayer_config() -> RelayerContractConfig {
    RelayerContractConfig {
        safe_factory: "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b".to_string(),
        safe_multisend: "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761".to_string(),
        ctf: "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045".to_string(),
        collateral: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
    }
}

/// Polygon Amoy testnet relayer configuration
pub fn amoy_relayer_config() -> RelayerContractConfig {
    RelayerContractConfig {
        safe_factory: "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b".to_string(),
        safe_multisend: "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761".to_string(),
        ctf: "0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB".to_string(),
        collateral: "0x9c4e1703476e875070ee25b56a58b008cfb8fa78".to_string(),
    }
}

/// Get relayer config for a chain ID
pub fn get_relayer_config(chain_id: u64) -> Option<RelayerContractConfig> {
    match chain_id {
        137 => Some(mainnet_relayer_config()),
        80002 => Some(amoy_relayer_config()),
        _ => None,
    }
}

/// Position data from the data API (internal use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PositionData {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub asset: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(deserialize_with = "deserialize_number_to_string")]
    pub size: String,
    pub redeemable: bool,
    pub mergeable: bool,
    pub title: String,
    pub outcome: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u32,
    /// Current price - 1.0 means winning, 0.0 means losing
    #[serde(rename = "curPrice", default)]
    pub cur_price: Option<f64>,
}

/// A position that can be redeemed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemablePosition {
    /// The condition ID of the market
    pub condition_id: String,
    /// The asset (token) ID
    pub asset: String,
    /// The size of the position
    pub size: String,
    /// The outcome name (e.g., "Yes", "No")
    pub outcome: String,
    /// The outcome index (0 or 1)
    pub outcome_index: u32,
    /// The market title
    pub title: String,
}
