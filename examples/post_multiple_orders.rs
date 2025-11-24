use alloy_signer_local::PrivateKeySigner;
use polymarket_rs::client::{AuthenticatedClient, TradingClient};
use polymarket_rs::orders::OrderBuilder;
use polymarket_rs::types::{
    CreateOrderOptions, OrderArgs, OrderType, PostOrderArgs, Side, SignatureType,
};
use polymarket_rs::Result;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Replace with your actual private key
    let private_key =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable not set");

    let signer = PrivateKeySigner::from_str(&private_key).expect("Invalid private key");

    let chain_id = 137; // Polygon Mainnet
    let host = "https://clob.polymarket.com";

    println!("Signer address: {}", signer.address());

    // Step 1: Create or derive API credentials
    println!("\n1. Creating/deriving API credentials...");
    let auth_client = AuthenticatedClient::new(host, signer.clone(), chain_id, None, None);
    let api_creds = auth_client.create_or_derive_api_key().await?;
    println!("API Key: {}", api_creds.api_key);
    println!("Successfully authenticated!");

    // Step 2: Create a trading client
    println!("\n2. Setting up trading client...");
    let order_builder = OrderBuilder::new(signer.clone(), Some(SignatureType::Eoa), None);

    let trading_client = TradingClient::new(
        host,
        signer.clone(),
        chain_id,
        api_creds.clone(),
        order_builder,
    );

    // Step 3: Create multiple orders
    println!("\n3. Creating multiple orders...");

    // Specific token IDs
    let token_id_1 = "";
    let token_id_2 = "";

    // Create order options (adjust tick_size and neg_risk as needed)
    let options = CreateOrderOptions::default()
        .tick_size(Decimal::from_str("0.01").unwrap())
        .neg_risk(false);

    // Create first order: BUY 10 tokens at 0.50
    let order_args_1 = OrderArgs::new(
        token_id_1,
        Decimal::from_str("0.50").unwrap(), // price
        Decimal::from_str("10.0").unwrap(), // size
        Side::Buy,
    );

    let signed_order_1 = trading_client.create_order(&order_args_1, None, None, options.clone())?;
    println!("Created order 1: BUY 10 @ 0.50");

    // Create second order: SELL 15 tokens at 0.75
    let order_args_2 = OrderArgs::new(
        token_id_2,
        Decimal::from_str("0.75").unwrap(), // price
        Decimal::from_str("15.0").unwrap(), // size
        Side::Sell,
    );

    let signed_order_2 = trading_client.create_order(&order_args_2, None, None, options.clone())?;
    println!("Created order 2: SELL 15 @ 0.75");

    // Create third order: BUY 5 tokens at 0.60
    let order_args_3 = OrderArgs::new(
        token_id_1,
        Decimal::from_str("0.60").unwrap(), // price
        Decimal::from_str("5.0").unwrap(),  // size
        Side::Buy,
    );

    let signed_order_3 = trading_client.create_order(&order_args_3, None, None, options)?;
    println!("Created order 3: BUY 5 @ 0.60");

    // Step 4: Post all orders at once
    println!("\n4. Posting multiple orders...");

    let results = trading_client
        .post_orders(&[
            PostOrderArgs::new(signed_order_1, OrderType::Gtc),
            PostOrderArgs::new(signed_order_2, OrderType::Gtc),
            PostOrderArgs::new(signed_order_3, OrderType::Gtc),
        ])
        .await?;

    println!("\n✅ Successfully posted {} orders!", results.len());

    for (i, result) in results.iter().enumerate() {
        println!("\nOrder {}:", i + 1);
        println!("  Order ID: {}", result.order_id.as_str());
        println!("  Status: {}", result.status);
        println!("  Success: {}", result.success);
        if !result.error_msg.is_empty() {
            println!("  Error: {}", result.error_msg);
        }
    }
    Ok(())
}
