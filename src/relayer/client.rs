//! Relayer Client for Polymarket
//!
//! This module provides a client for interacting with Polymarket's Polygon relayer
//! infrastructure, enabling gasless transactions for Safe wallets.

use crate::error::{Error, Result};
use crate::signing::EthSigner;
use alloy_primitives::{hex, keccak256, B256};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use super::ctf::CtfEncoder;
use super::types::*;

type HmacSha256 = Hmac<Sha256>;

/// Relayer Client for Safe wallet transactions
///
/// This client allows you to execute gasless transactions through Polymarket's
/// relayer infrastructure. It supports:
/// - Safe wallet deployment
/// - CTF operations (split, merge, redeem positions)
/// - Token approvals
/// - Custom transaction execution
pub struct RelayerClient {
    http_client: Client,
    relayer_url: String,
    chain_id: u64,
    signer: Option<Box<dyn EthSigner>>,
    builder_creds: Option<BuilderApiCreds>,
    contract_config: RelayerContractConfig,
}

impl RelayerClient {
    /// Create a new RelayerClient
    ///
    /// # Arguments
    /// * `relayer_url` - The relayer API URL (e.g., "https://relayer-v2.polymarket.com")
    /// * `chain_id` - The chain ID (137 for Polygon, 80002 for Amoy)
    /// * `signer` - Optional Ethereum signer for transaction signing
    /// * `builder_creds` - Optional Builder API credentials for authentication
    pub fn new(
        relayer_url: impl Into<String>,
        chain_id: u64,
        signer: Option<impl EthSigner + 'static>,
        builder_creds: Option<BuilderApiCreds>,
    ) -> Result<Self> {
        let contract_config = get_relayer_config(chain_id)
            .ok_or_else(|| Error::Config(format!("Unsupported chain_id: {}", chain_id)))?;

        let url = relayer_url.into();
        let url = if url.ends_with('/') {
            url[..url.len() - 1].to_string()
        } else {
            url
        };

        Ok(Self {
            http_client: Client::new(),
            relayer_url: url,
            chain_id,
            signer: signer.map(|s| Box::new(s) as Box<dyn EthSigner>),
            builder_creds,
            contract_config,
        })
    }

    /// Get the expected Safe wallet address for the signer
    pub fn get_expected_safe(&self) -> Result<String> {
        let signer = self.require_signer()?;
        // Normalize address to lowercase hex for consistency with SDK
        let signer_address = format!("0x{}", hex::encode(signer.address().as_slice()));
        Ok(derive_safe_address(
            &signer_address,
            &self.contract_config.safe_factory,
        ))
    }

    /// Check if a Safe wallet is deployed
    pub async fn get_deployed(&self, safe_address: &str) -> Result<bool> {
        let url = format!("{}/deployed?address={}", self.relayer_url, safe_address);
        let response: DeployedResponse = self.http_client.get(&url).send().await?.json().await?;
        Ok(response.deployed)
    }

    /// Get the nonce for signing transactions
    pub async fn get_nonce(&self, address: &str, tx_type: TransactionType) -> Result<String> {
        let url = format!(
            "{}/nonce?address={}&type={}",
            self.relayer_url,
            address,
            tx_type.as_str()
        );
        let response: NonceResponse = self.http_client.get(&url).send().await?.json().await?;
        Ok(response.nonce)
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Vec<RelayerTransaction>> {
        let url = format!("{}/transaction?id={}", self.relayer_url, transaction_id);
        let response: Vec<RelayerTransaction> =
            self.http_client.get(&url).send().await?.json().await?;
        Ok(response)
    }

    /// Deploy a Safe wallet
    ///
    /// This creates a new Safe wallet for the signer. The wallet must not already be deployed.
    pub async fn deploy(&self) -> Result<RelayerSubmitResponse> {
        let signer = self.require_signer()?;
        self.require_builder_creds()?;

        let safe_address = self.get_expected_safe()?;
        let deployed = self.get_deployed(&safe_address).await?;

        if deployed {
            return Err(Error::Config(format!(
                "Safe {} is already deployed",
                safe_address
            )));
        }

        // Normalize address to lowercase hex for consistency with SDK
        let from_address = format!("0x{}", hex::encode(signer.address().as_slice()));

        // Create the struct hash for Safe creation
        let struct_hash = create_safe_create_struct_hash(
            &self.contract_config.safe_factory,
            self.chain_id,
            ZERO_ADDRESS,
            "0",
            ZERO_ADDRESS,
        );

        // Sign the struct hash
        let signature = sign_eip712_struct_hash(signer, &struct_hash)?;

        let request = TransactionRequest {
            tx_type: TransactionType::SafeCreate.as_str().to_string(),
            from: from_address,
            to: self.contract_config.safe_factory.clone(),
            proxy_wallet: safe_address,
            data: "0x".to_string(),
            signature,
            value: None,
            nonce: None,
            signature_params: Some(SignatureParams::for_safe_create()),
            metadata: None,
        };

        self.submit_transaction(request).await
    }

    /// Execute transactions through the Safe wallet
    ///
    /// # Arguments
    /// * `transactions` - List of transactions to execute
    /// * `metadata` - Optional metadata (max 500 characters)
    pub async fn execute(
        &self,
        transactions: Vec<SafeTransaction>,
        metadata: Option<&str>,
    ) -> Result<RelayerSubmitResponse> {
        let signer = self.require_signer()?;
        self.require_builder_creds()?;

        if transactions.is_empty() {
            return Err(Error::InvalidParameter("No transactions provided".into()));
        }

        let safe_address = self.get_expected_safe()?;
        let deployed = self.get_deployed(&safe_address).await?;

        if !deployed {
            return Err(Error::Config(format!(
                "Safe {} is not deployed. Call deploy() first.",
                safe_address
            )));
        }

        // Normalize address to lowercase hex for consistency with SDK
        let from_address = format!("0x{}", hex::encode(signer.address().as_slice()));
        // Query nonce using EOA address - the relayer internally derives the Safe
        // and returns the Safe's nonce (matching SDK behavior)
        let nonce = self.get_nonce(&from_address, TransactionType::Safe).await?;

        // Aggregate transactions if more than one
        let (final_tx, operation) = if transactions.len() == 1 {
            let tx = &transactions[0];
            (tx.clone(), tx.operation)
        } else {
            (
                aggregate_transactions(&transactions, &self.contract_config.safe_multisend),
                OperationType::DelegateCall,
            )
        };

        // Create the struct hash for Safe execution
        let struct_hash = create_safe_struct_hash(
            self.chain_id,
            &safe_address,
            &final_tx.to,
            &final_tx.value,
            &final_tx.data,
            operation,
            "0",
            "0",
            "0",
            ZERO_ADDRESS,
            ZERO_ADDRESS,
            &nonce,
        );

        // Sign the struct hash
        let signature = sign_eip712_struct_hash(signer, &struct_hash)?;

        let request = TransactionRequest {
            tx_type: TransactionType::Safe.as_str().to_string(),
            from: from_address,
            to: final_tx.to,
            proxy_wallet: safe_address,
            data: final_tx.data,
            signature,
            value: Some(final_tx.value),
            nonce: Some(nonce),
            signature_params: Some(SignatureParams::for_safe_execution(operation)),
            metadata: metadata.map(|s| s.to_string()),
        };

        self.submit_transaction(request).await
    }

    /// Redeem positions after market resolution
    ///
    /// This redeems winning conditional tokens for collateral after a market has been resolved.
    ///
    /// # Arguments
    /// * `condition_id` - The condition ID of the resolved market
    /// * `index_sets` - The index sets to redeem (typically [1, 2] for YES/NO markets)
    /// * `metadata` - Optional metadata
    pub async fn redeem_positions(
        &self,
        condition_id: &str,
        index_sets: Vec<u32>,
        metadata: Option<&str>,
    ) -> Result<RelayerSubmitResponse> {
        let data = CtfEncoder::encode_redeem_positions(
            &self.contract_config.collateral,
            condition_id,
            index_sets,
        );

        let tx = SafeTransaction::new(&self.contract_config.ctf, data);
        self.execute(vec![tx], metadata).await
    }

    /// Split collateral into conditional tokens
    ///
    /// # Arguments
    /// * `condition_id` - The condition ID
    /// * `amount` - Amount of collateral to split (in smallest units)
    /// * `metadata` - Optional metadata
    pub async fn split_position(
        &self,
        condition_id: &str,
        amount: &str,
        metadata: Option<&str>,
    ) -> Result<RelayerSubmitResponse> {
        let data = CtfEncoder::encode_split_position(
            &self.contract_config.collateral,
            condition_id,
            amount,
        );

        let tx = SafeTransaction::new(&self.contract_config.ctf, data);
        self.execute(vec![tx], metadata).await
    }

    /// Merge conditional tokens back into collateral
    ///
    /// # Arguments
    /// * `condition_id` - The condition ID
    /// * `amount` - Amount to merge (in smallest units)
    /// * `metadata` - Optional metadata
    pub async fn merge_positions(
        &self,
        condition_id: &str,
        amount: &str,
        metadata: Option<&str>,
    ) -> Result<RelayerSubmitResponse> {
        let data = CtfEncoder::encode_merge_positions(
            &self.contract_config.collateral,
            condition_id,
            amount,
        );

        let tx = SafeTransaction::new(&self.contract_config.ctf, data);
        self.execute(vec![tx], metadata).await
    }

    /// Wait for a transaction to reach a terminal state
    ///
    /// # Arguments
    /// * `transaction_id` - The transaction ID to wait for
    /// * `max_polls` - Maximum number of poll attempts (default: 30)
    /// * `poll_interval_ms` - Interval between polls in milliseconds (default: 2000)
    pub async fn wait_for_transaction(
        &self,
        transaction_id: &str,
        max_polls: Option<u32>,
        poll_interval_ms: Option<u64>,
    ) -> Result<Option<RelayerTransaction>> {
        let max_polls = max_polls.unwrap_or(30);
        let poll_interval = std::time::Duration::from_millis(poll_interval_ms.unwrap_or(2000));

        for _ in 0..max_polls {
            let transactions = self.get_transaction(transaction_id).await?;

            if let Some(tx) = transactions.into_iter().next() {
                if let Some(state) = tx.get_state() {
                    if state.is_success() {
                        return Ok(Some(tx));
                    }
                    if state == RelayerTransactionState::Failed
                        || state == RelayerTransactionState::Invalid
                    {
                        return Err(Error::Api {
                            status: 400,
                            message: format!(
                                "Transaction {} failed with state {:?}",
                                transaction_id, state
                            ),
                        });
                    }
                }
            }

            tokio::time::sleep(poll_interval).await;
        }

        Ok(None)
    }

    /// Get redeemable positions for a user from the data API
    ///
    /// This fetches positions that are marked as redeemable by the API.
    /// The API filters for positions in resolved markets that can be redeemed.
    ///
    /// # Arguments
    /// * `data_api_url` - The data API URL (e.g., "https://data-api.polymarket.com")
    /// * `user_address` - The user's wallet address (Safe wallet address)
    ///
    /// # Returns
    /// A list of redeemable positions with their condition IDs and sizes
    pub async fn get_redeemable_positions(
        &self,
        data_api_url: &str,
        user_address: &str,
    ) -> Result<Vec<RedeemablePosition>> {
        let url = format!(
            "{}/positions?user={}&redeemable=true&sizeThreshold=0.1&limit=100&offset=0&sortBy=CURRENT&sortDirection=DESC",
            data_api_url, user_address
        );
        let response: Vec<PositionData> = self.http_client.get(&url).send().await?.json().await?;

        let redeemable: Vec<RedeemablePosition> = response
            .into_iter()
            // Only include positions with currentValue > 0 (winning positions worth redeeming)
            .filter(|p| p.current_value > 0.0)
            .map(|p| RedeemablePosition {
                condition_id: p.condition_id,
                asset: p.asset,
                size: p.size,
                outcome: p.outcome,
                outcome_index: p.outcome_index,
                title: p.title,
                current_value: p.current_value,
            })
            .collect();

        Ok(redeemable)
    }

    /// Redeem all redeemable positions for the current Safe wallet
    ///
    /// This is a convenience method that:
    /// 1. Gets the Safe wallet address
    /// 2. Fetches all redeemable positions
    /// 3. Redeems each position
    ///
    /// # Arguments
    /// * `data_api_url` - The data API URL
    ///
    /// # Returns
    /// A list of (condition_id, transaction_response) tuples for each redeemed position
    pub async fn redeem_all_positions(
        &self,
        data_api_url: &str,
    ) -> Result<Vec<(String, RelayerSubmitResponse)>> {
        let safe_address = self.get_expected_safe()?;
        let redeemable = self
            .get_redeemable_positions(data_api_url, &safe_address)
            .await?;

        let mut results = Vec::new();

        for position in redeemable {
            // Calculate the correct index set based on outcome_index
            // index_set is a bitmask: 1 << outcome_index
            // outcome_index 0 (YES) -> index_set 1 (binary: 01)
            // outcome_index 1 (NO)  -> index_set 2 (binary: 10)
            let index_set = 1u32 << position.outcome_index;

            let result = self
                .redeem_positions(
                    &position.condition_id,
                    vec![index_set],
                    Some(&format!("Redeem: {}", position.title)),
                )
                .await?;

            results.push((position.condition_id, result));
        }

        Ok(results)
    }

    /// Get the contract configuration
    pub fn contract_config(&self) -> &RelayerContractConfig {
        &self.contract_config
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    // Private helper methods

    fn require_signer(&self) -> Result<&dyn EthSigner> {
        self.signer
            .as_ref()
            .map(|s| s.as_ref())
            .ok_or_else(|| Error::AuthRequired("Signer is required for this operation".into()))
    }

    fn require_builder_creds(&self) -> Result<&BuilderApiCreds> {
        self.builder_creds.as_ref().ok_or_else(|| {
            Error::AuthRequired("Builder credentials are required for this operation".into())
        })
    }

    async fn submit_transaction(
        &self,
        request: TransactionRequest,
    ) -> Result<RelayerSubmitResponse> {
        let builder_creds = self.require_builder_creds()?;

        let body = serde_json::to_string(&request)?;
        let headers = generate_builder_headers(builder_creds, "POST", "/submit", Some(&body))?;

        let response = self
            .http_client
            .post(format!("{}/submit", self.relayer_url))
            .header("POLY_BUILDER_API_KEY", &headers.api_key)
            .header("POLY_BUILDER_SIGNATURE", &headers.signature)
            .header("POLY_BUILDER_TIMESTAMP", &headers.timestamp)
            .header("POLY_BUILDER_PASSPHRASE", &headers.passphrase)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Api { status, message });
        }

        let result: RelayerSubmitResponse = response.json().await?;
        Ok(result)
    }
}

// Helper structs and functions

struct BuilderHeaders {
    api_key: String,
    signature: String,
    timestamp: String,
    passphrase: String,
}

fn generate_builder_headers(
    creds: &BuilderApiCreds,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<BuilderHeaders> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Signing(e.to_string()))?
        .as_secs();

    let timestamp_str = timestamp.to_string();
    let body_str = body.unwrap_or("");
    let message = format!("{}{}{}{}", timestamp_str, method, path, body_str);

    // Use STANDARD base64 decoding for the secret (matching TypeScript SDK)
    // TypeScript uses Buffer.from(secret, "base64") which is standard base64
    let secret_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &creds.secret)
            .or_else(|_| {
                // Fallback: try URL-safe if standard fails (for flexibility)
                base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, &creds.secret)
            })
            .map_err(|e| Error::Signing(format!("Failed to decode secret: {}", e)))?;

    let mut mac = HmacSha256::new_from_slice(&secret_bytes)
        .map_err(|e| Error::Signing(format!("HMAC error: {}", e)))?;
    mac.update(message.as_bytes());

    // Use URL-safe base64 encoding for the signature (matching TypeScript SDK)
    // TypeScript converts '+' to '-' and '/' to '_' but keeps '=' padding
    let signature = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        mac.finalize().into_bytes(),
    );

    // Convert to URL-safe: '+' -> '-', '/' -> '_'
    let signature = signature.replace('+', "-").replace('/', "_");

    Ok(BuilderHeaders {
        api_key: creds.key.clone(),
        signature,
        timestamp: timestamp_str,
        passphrase: creds.passphrase.clone(),
    })
}

/// Derive Safe wallet address from signer address
pub fn derive_safe_address(address: &str, safe_factory: &str) -> String {
    let address = address.to_lowercase();
    let address = if address.starts_with("0x") {
        &address[2..]
    } else {
        &address
    };

    // Encode the address for salt calculation: keccak256(abi.encode(address))
    let mut padded_address = vec![0u8; 12]; // 12 bytes of padding
    padded_address.extend(hex::decode(address).unwrap_or_default());
    let salt = keccak256(&padded_address);

    // CREATE2 address calculation
    let factory = safe_factory.to_lowercase();
    let factory = if factory.starts_with("0x") {
        &factory[2..]
    } else {
        &factory
    };

    let init_code_hash = if SAFE_INIT_CODE_HASH.starts_with("0x") {
        &SAFE_INIT_CODE_HASH[2..]
    } else {
        SAFE_INIT_CODE_HASH
    };

    let mut data = vec![0xff];
    data.extend(hex::decode(factory).unwrap_or_default());
    data.extend(salt.as_slice());
    data.extend(hex::decode(init_code_hash).unwrap_or_default());

    let hash = keccak256(&data);
    format!("0x{}", hex::encode(&hash[12..]))
}

/// Create struct hash for Safe creation
fn create_safe_create_struct_hash(
    safe_factory: &str,
    chain_id: u64,
    payment_token: &str,
    payment: &str,
    payment_receiver: &str,
) -> B256 {
    // CreateProxy type hash
    let type_hash =
        keccak256(b"CreateProxy(address paymentToken,uint256 payment,address paymentReceiver)");

    // Encode payment token
    let payment_token_bytes = encode_address(payment_token);
    // Encode payment
    let payment_bytes = encode_uint256(payment);
    // Encode payment receiver
    let payment_receiver_bytes = encode_address(payment_receiver);

    // struct hash = keccak256(typeHash || encoded_values)
    let mut struct_data = type_hash.to_vec();
    struct_data.extend(&payment_token_bytes);
    struct_data.extend(&payment_bytes);
    struct_data.extend(&payment_receiver_bytes);
    let struct_hash = keccak256(&struct_data);

    // Domain separator
    let domain_separator = make_domain_separator(SAFE_FACTORY_NAME, safe_factory, chain_id);

    // Final hash = keccak256(0x19 || 0x01 || domainSeparator || structHash)
    let mut final_data = vec![0x19, 0x01];
    final_data.extend(domain_separator.as_slice());
    final_data.extend(struct_hash.as_slice());
    keccak256(&final_data)
}

/// Create struct hash for Safe transaction
fn create_safe_struct_hash(
    chain_id: u64,
    safe: &str,
    to: &str,
    value: &str,
    data: &str,
    operation: OperationType,
    safe_tx_gas: &str,
    base_gas: &str,
    gas_price: &str,
    gas_token: &str,
    refund_receiver: &str,
    nonce: &str,
) -> B256 {
    // SafeTx type hash
    let type_hash = keccak256(
        b"SafeTx(address to,uint256 value,bytes data,uint8 operation,uint256 safeTxGas,uint256 baseGas,uint256 gasPrice,address gasToken,address refundReceiver,uint256 nonce)",
    );

    // Encode data hash
    let data_bytes = if data.starts_with("0x") {
        hex::decode(&data[2..]).unwrap_or_default()
    } else {
        hex::decode(data).unwrap_or_default()
    };
    let data_hash = keccak256(&data_bytes);

    // Build struct hash
    let mut struct_data = type_hash.to_vec();
    struct_data.extend(encode_address(to));
    struct_data.extend(encode_uint256(value));
    struct_data.extend(data_hash.as_slice());
    struct_data.extend(encode_uint8(operation as u8));
    struct_data.extend(encode_uint256(safe_tx_gas));
    struct_data.extend(encode_uint256(base_gas));
    struct_data.extend(encode_uint256(gas_price));
    struct_data.extend(encode_address(gas_token));
    struct_data.extend(encode_address(refund_receiver));
    struct_data.extend(encode_uint256(nonce));

    let struct_hash = keccak256(&struct_data);

    // Domain separator for Safe (no name, just chainId and verifyingContract)
    let domain_separator = make_safe_domain_separator(safe, chain_id);

    // Final hash
    let mut final_data = vec![0x19, 0x01];
    final_data.extend(domain_separator.as_slice());
    final_data.extend(struct_hash.as_slice());
    keccak256(&final_data)
}

fn make_domain_separator(name: &str, verifying_contract: &str, chain_id: u64) -> B256 {
    let type_hash =
        keccak256(b"EIP712Domain(string name,address verifyingContract,uint256 chainId)");
    let name_hash = keccak256(name.as_bytes());

    let mut data = type_hash.to_vec();
    data.extend(name_hash.as_slice());
    data.extend(encode_address(verifying_contract));
    data.extend(encode_uint256(&chain_id.to_string()));

    keccak256(&data)
}

fn make_safe_domain_separator(safe: &str, chain_id: u64) -> B256 {
    // Safe uses a domain separator with just chainId and verifyingContract (no name)
    let type_hash = keccak256(b"EIP712Domain(uint256 chainId,address verifyingContract)");

    let mut data = type_hash.to_vec();
    data.extend(encode_uint256(&chain_id.to_string()));
    data.extend(encode_address(safe));

    keccak256(&data)
}

fn encode_address(addr: &str) -> [u8; 32] {
    let addr = if addr.starts_with("0x") {
        &addr[2..]
    } else {
        addr
    };

    let mut result = [0u8; 32];
    let bytes = hex::decode(addr).unwrap_or_default();
    if bytes.len() <= 20 {
        result[32 - bytes.len()..].copy_from_slice(&bytes);
    }
    result
}

fn encode_uint256(value: &str) -> [u8; 32] {
    let value = value.parse::<u128>().unwrap_or(0);
    let mut result = [0u8; 32];
    result[16..].copy_from_slice(&value.to_be_bytes());
    result
}

fn encode_uint8(value: u8) -> [u8; 32] {
    let mut result = [0u8; 32];
    result[31] = value;
    result
}

fn sign_eip712_struct_hash(signer: &dyn EthSigner, hash: &B256) -> Result<String> {
    // Sign the EIP-712 hash using signMessage (eth_sign style)
    // This adds EIP-191 prefix internally: keccak256("\x19Ethereum Signed Message:\n32" + hash)
    // Safe contract expects v >= 31 for eth_sign style signatures
    let signature = signer
        .sign_message_sync(hash.as_slice())
        .map_err(|e| Error::Signing(e.to_string()))?;

    // Adjust v-value for Safe contract's eth_sign verification
    // Safe contract: when v >= 31, it computes: ecrecover(keccak256("\x19Ethereum..." + dataHash), v - 4, r, s)
    // This matches the EIP-191 prefix that signMessage already added
    let mut sig_bytes = signature.as_bytes().to_vec();
    let v = sig_bytes[64];
    sig_bytes[64] = match v {
        0 => 31,    // 0 -> 31 (for eth_sign)
        1 => 32,    // 1 -> 32 (for eth_sign)
        27 => 31,   // 27 -> 31 (27 + 4 = 31)
        28 => 32,   // 28 -> 32 (28 + 4 = 32)
        _ => v + 4, // Generic case
    };

    Ok(format!("0x{}", hex::encode(sig_bytes)))
}

/// Aggregate multiple transactions into a single multisend transaction
fn aggregate_transactions(
    transactions: &[SafeTransaction],
    multisend_address: &str,
) -> SafeTransaction {
    // Encode each transaction for multisend
    let mut encoded_txs = Vec::new();

    for tx in transactions {
        // operation (1 byte) + to (20 bytes) + value (32 bytes) + dataLength (32 bytes) + data
        let to_bytes = hex::decode(tx.to.trim_start_matches("0x")).unwrap_or_default();
        let value: u128 = tx.value.parse().unwrap_or(0);
        let data_bytes = hex::decode(tx.data.trim_start_matches("0x")).unwrap_or_default();

        encoded_txs.push(tx.operation as u8);
        // Pad to address to 20 bytes
        let mut to_padded = vec![0u8; 20 - to_bytes.len().min(20)];
        to_padded.extend(&to_bytes[..to_bytes.len().min(20)]);
        encoded_txs.extend(&to_padded);
        // Value as 32 bytes big-endian
        let mut value_bytes = vec![0u8; 16];
        value_bytes.extend(&value.to_be_bytes());
        encoded_txs.extend(&value_bytes);
        // Data length as 32 bytes big-endian
        let data_len = data_bytes.len() as u128;
        let mut len_bytes = vec![0u8; 16];
        len_bytes.extend(&data_len.to_be_bytes());
        encoded_txs.extend(&len_bytes);
        // Data
        encoded_txs.extend(&data_bytes);
    }

    // Create multisend call: multiSend(bytes transactions)
    // Function selector: 0x8d80ff0a
    let selector = hex::decode("8d80ff0a").unwrap();

    // Encode as bytes: offset (32 bytes) + length (32 bytes) + data (padded to 32 bytes)
    let offset: u128 = 32;
    let length = encoded_txs.len() as u128;

    let mut multisend_data = selector;
    // Offset
    let mut offset_bytes = vec![0u8; 16];
    offset_bytes.extend(&offset.to_be_bytes());
    multisend_data.extend(&offset_bytes);
    // Length
    let mut len_bytes = vec![0u8; 16];
    len_bytes.extend(&length.to_be_bytes());
    multisend_data.extend(&len_bytes);
    // Data (padded to 32-byte boundary)
    multisend_data.extend(&encoded_txs);
    let padding = (32 - (encoded_txs.len() % 32)) % 32;
    multisend_data.extend(vec![0u8; padding]);

    SafeTransaction {
        to: multisend_address.to_string(),
        operation: OperationType::DelegateCall,
        data: format!("0x{}", hex::encode(&multisend_data)),
        value: "0".to_string(),
    }
}
