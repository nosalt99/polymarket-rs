//! Example: Redeem Positions via Relayer
//!
//! This example demonstrates how to use the Polymarket Relayer Client to redeem
//! winning positions after a market has been resolved.
//!
//! The relayer allows for gasless transactions - Polymarket pays the gas fees.
//!
//! ## Prerequisites
//!
//! 1. A deployed Safe wallet (the relayer will create one if needed)
//! 2. **Builder API credentials** - These are different from CLOB API credentials!
//!    - Go to https://polymarket.com/settings?tab=builder
//!    - Click "+ Create New" in the Builder Keys section
//!    - Save your apiKey, secret, and passphrase
//! 3. Winning positions in a resolved market
//!
//! ## Environment Variables
//!
//! - `PRIVATE_KEY`: Your Ethereum private key
//! - `POLY_API_KEY`: Your **Builder** API key (from Builder Keys, NOT CLOB API)
//! - `POLY_API_SECRET`: Your **Builder** API secret
//! - `POLY_PASSPHRASE`: Your **Builder** API passphrase
//!
//! ## Usage
//!
//! ```bash
//! PRIVATE_KEY=0x... \
//! POLY_API_KEY=... \
//! POLY_API_SECRET=... \
//! POLY_PASSPHRASE=... \
//! cargo run --example redeem_positions
//! ```

use alloy_signer_local::PrivateKeySigner;
use polymarket_rs::relayer::{BuilderApiCreds, CtfEncoder, RelayerClient, SafeTransaction};
use polymarket_rs::Result;
use std::str::FromStr;

const DATA_API_MAINNET: &str = "https://data-api.polymarket.com";
const DATA_API_TESTNET: &str = "https://data-api.polymarket.com"; // Same for testnet

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    let private_key =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable not set");

    // Create the signer
    let signer = PrivateKeySigner::from_str(&private_key).expect("Invalid private key");

    println!("Signer address: {}", signer.address());

    // Get Builder API credentials from environment variables
    let builder_creds = BuilderApiCreds::from_env().expect(
        "Builder API credentials not set. Set POLY_API_KEY, POLY_API_SECRET, and POLY_PASSPHRASE",
    );

    // Initialize the Relayer Client
    // Use 137 for Polygon Mainnet, 80002 for Amoy Testnet
    let chain_id = std::env::var("CHAIN_ID")
        .unwrap_or_else(|_| "137".to_string())
        .parse::<u64>()
        .unwrap_or(137);

    let relayer_url = if chain_id == 137 {
        "https://relayer-v2.polymarket.com"
    } else {
        "https://relayer-v2-staging.polymarket.dev"
    };

    let data_api_url = if chain_id == 137 {
        DATA_API_MAINNET
    } else {
        DATA_API_TESTNET
    };

    println!("Using chain ID: {}", chain_id);
    println!("Relayer URL: {}", relayer_url);
    println!("Data API URL: {}", data_api_url);

    let client = RelayerClient::new(relayer_url, chain_id, Some(signer), Some(builder_creds))?;

    // Get the expected Safe wallet address
    let safe_address = client.get_expected_safe()?;
    println!("Safe wallet address: {}", safe_address);

    // Check if the Safe is deployed
    let deployed = client.get_deployed(&safe_address).await?;
    println!("Safe deployed: {}", deployed);

    if !deployed {
        println!("\n=== Deploying Safe Wallet ===");
        let deploy_result = client.deploy().await?;
        println!("Deploy transaction ID: {}", deploy_result.transaction_id);

        // Wait for deployment to complete
        println!("Waiting for deployment...");
        let tx = client
            .wait_for_transaction(&deploy_result.transaction_id, Some(30), Some(2000))
            .await?;

        if let Some(tx) = tx {
            println!("Safe deployed successfully!");
            println!("Transaction hash: {:?}", tx.transaction_hash);
        } else {
            println!("Deployment is taking longer than expected. Check the transaction ID.");
            return Ok(());
        }
    }

    // Get all redeemable positions and redeem them
    println!("\n=== Fetching Redeemable Positions ===");

    let redeemable_positions = client
        .get_redeemable_positions(data_api_url, &safe_address)
        .await?;

    if redeemable_positions.is_empty() {
        println!("No redeemable positions found for this wallet.");
        println!("\nRedeemable positions are from markets that have been resolved.");
        println!("Make sure you have winning positions in resolved markets.");
    } else {
        println!(
            "Found {} redeemable position(s):",
            redeemable_positions.len()
        );

        for (i, pos) in redeemable_positions.iter().enumerate() {
            println!(
                "\n  {}. {} - {} (size: {})",
                i + 1,
                pos.title,
                pos.outcome,
                pos.size
            );
            println!("     Condition ID: {}", pos.condition_id);
        }

        println!("\n=== Redeeming All Positions ===");

        let mut success_count = 0;
        let mut fail_count = 0;
        let total = redeemable_positions.len();

        for (i, pos) in redeemable_positions.iter().enumerate() {
            println!(
                "\n[{}/{}] Redeeming: {} - {}",
                i + 1,
                total,
                pos.title,
                pos.outcome
            );

            // Calculate the correct index set based on outcome_index
            // index_set is a bitmask: 1 << outcome_index
            // outcome_index 0 (YES) -> index_set 1 (binary: 01)
            // outcome_index 1 (NO)  -> index_set 2 (binary: 10)
            let index_set = 1u32 << pos.outcome_index;

            match client
                .redeem_positions(
                    &pos.condition_id,
                    vec![index_set],
                    Some(&format!("Redeem: {}", pos.title)),
                )
                .await
            {
                Ok(result) => {
                    println!("  ✓ Transaction submitted!");
                    println!("  Transaction ID: {}", result.transaction_id);
                    if let Some(hash) = result.transaction_hash {
                        println!("  Transaction hash: {}", hash);
                    }
                    success_count += 1;
                }
                Err(e) => {
                    println!("  ✗ Failed to redeem: {}", e);
                    fail_count += 1;
                    // Continue with next position instead of stopping
                }
            }

            // Small delay between transactions to avoid rate limiting
            if i < total - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        println!("\n=== Redemption Summary ===");
        println!("Total positions: {}", total);
        println!("Successful: {}", success_count);
        println!("Failed: {}", fail_count);

        if success_count > 0 {
            println!("\nTransactions are being processed by the relayer.");
            println!("You can check the status on Polygonscan.");
        }
    }

    // Show additional information
    println!("\n=== Other Available Operations ===");

    // 1. Split position (convert USDC to YES/NO tokens)
    println!("\n1. Split Position:");
    println!("   client.split_position(condition_id, amount, metadata).await?");
    println!("   - Converts USDC to YES and NO tokens");
    println!("   - Requires USDC approval to CTF contract first");

    // 2. Merge positions (convert YES+NO tokens back to USDC)
    println!("\n2. Merge Positions:");
    println!("   client.merge_positions(condition_id, amount, metadata).await?");
    println!("   - Converts equal amounts of YES and NO tokens back to USDC");

    // 3. Custom transaction execution
    println!("\n3. Custom Transactions:");
    println!("   Use client.execute(transactions, metadata) for custom operations");

    // Example of creating a custom approval transaction
    let ctf_address = client.contract_config().ctf.clone();
    let collateral_address = client.contract_config().collateral.clone();

    println!("\n   Example - Approve USDC for CTF:");
    let approve_data = CtfEncoder::encode_approve_max(&ctf_address);
    let _approve_tx = SafeTransaction::new(&collateral_address, approve_data);
    println!("   let tx = SafeTransaction::new(&collateral, encode_approve_max(&ctf));");
    println!("   client.execute(vec![tx], Some(\"Approve USDC\")).await?");

    println!("\n=== Contract Addresses ===");
    println!("CTF: {}", client.contract_config().ctf);
    println!("Collateral (USDC): {}", client.contract_config().collateral);
    println!("Safe Factory: {}", client.contract_config().safe_factory);
    println!(
        "Safe Multisend: {}",
        client.contract_config().safe_multisend
    );

    println!("\nDone!");
    Ok(())
}
