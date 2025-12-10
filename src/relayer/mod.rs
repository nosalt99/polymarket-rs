//! Polymarket Relayer Client Module
//!
//! This module provides a client for interacting with Polymarket's Polygon relayer
//! infrastructure, enabling gasless transactions for Safe wallets.
//!
//! # Features
//!
//! - **Gasless Transactions**: Polymarket pays for all gas fees
//! - **Safe Wallet Support**: Deploy and manage Gnosis Safe wallets
//! - **CTF Operations**: Split, merge, and redeem positions
//! - **Token Approvals**: Set allowances for trading tokens
//!
//! # Example
//!
//! ```no_run
//! use polymarket_rs::relayer::{RelayerClient, BuilderApiCreds};
//! use alloy_signer_local::PrivateKeySigner;
//! use std::str::FromStr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let signer = PrivateKeySigner::from_str("your-private-key")?;
//! let builder_creds = BuilderApiCreds::new(
//!     "your-api-key".to_string(),
//!     "your-secret".to_string(),
//!     "your-passphrase".to_string(),
//! );
//!
//! let client = RelayerClient::new(
//!     "https://relayer-v2.polymarket.com",
//!     137, // Polygon mainnet
//!     Some(signer),
//!     Some(builder_creds),
//! )?;
//!
//! // Deploy a Safe wallet
//! let deploy_result = client.deploy().await?;
//! let tx = client.wait_for_transaction(&deploy_result.transaction_id, None, None).await?;
//!
//! // Redeem positions after market resolution
//! let condition_id = "0x...";
//! let redeem_result = client.redeem_positions(condition_id, vec![1, 2], Some("Redeem positions")).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod ctf;
mod types;

pub use client::{derive_safe_address, RelayerClient};
pub use ctf::CtfEncoder;
pub use types::*;
