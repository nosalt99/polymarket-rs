# polymarket-rs

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

> [!NOTE]
> This library is under active development and considered alpha quality.
>
> - Breaking changes may occur in future updates without prior notice
> - API signatures, types, and module structures are subject to change
> - Not recommended for production use yet
> - Use at your own risk

A modern, type-safe Rust client library for the [Polymarket](https://polymarket.com) CLOB (Central Limit Order Book) and Data API.

This project is a complete rewrite of [polymarket-rs-client](https://github.com/TechieBoy/polymarket-rs-client) with improved ergonomics, additional API methods, and removal of generic type parameters for a cleaner API surface.

## Features

- **Full Authentication Support** - L1 (EIP-712) and L2 (HMAC) authentication
- **Type-Safe** - Strongly typed IDs (`TokenId`, `OrderId`, `ConditionId`) prevent mistakes
- **Builder Pattern** - Fluent APIs for configuration and order creation
- **Async/Await** - Built on `tokio` for high-performance async operations
- **Decimal Precision** - Accurate financial calculations with `rust_decimal`
- **Modular Design** - Separated clients for different operations
- **Zero Panics** - Comprehensive error handling with custom `Result` types

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
polymarket-rs = "0.1.0"
```

## Quick Start

### Public Market Data (No Authentication)

```rust
use polymarket_rs::{PublicClient, TokenId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PublicClient::new("https://clob.polymarket.com");

    let token_id = TokenId::new("21742633143463906290569050155826241533067272736897614950488156847949938836455");

    // Get midpoint price
    let midpoint = client.get_midpoint(&token_id).await?;
    println!("Midpoint: {}", midpoint.mid);

    // Get order book
    let book = client.get_order_book(&token_id).await?;
    println!("Best bid: {:?}", book.bids.first());
    println!("Best ask: {:?}", book.asks.first());

    Ok(())
}
```

### Authenticated Trading

```rust
use alloy_signer_local::PrivateKeySigner;
use polymarket_rs::{
    AuthenticatedClient, TradingClient, OrderBuilder,
    OrderArgs, Side, OrderType, SignatureType,
    CreateOrderOptions,
};
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load private key from environment
    let private_key = std::env::var("PRIVATE_KEY")?;
    let signer = PrivateKeySigner::from_str(&private_key)?;

    let chain_id = 137; // Polygon Mainnet
    let host = "https://clob.polymarket.com";

    // Step 1: Create or derive API credentials
    let auth_client = AuthenticatedClient::new(
        host,
        signer.clone(),
        chain_id,
        None,
    );

    let api_creds = auth_client.create_or_derive_api_key().await?;
    println!("Authenticated with API key: {}", api_creds.api_key);

    // Step 2: Create trading client
    let order_builder = OrderBuilder::new(
        signer.clone(),
        Some(SignatureType::Eoa),
        None,
    );

    let trading_client = TradingClient::new(
        host,
        signer,
        chain_id,
        api_creds,
        order_builder,
    );

    // Step 3: Create and post a limit order
    let order_args = OrderArgs::new(
        "token_id_here",
        Decimal::from_str("0.50")?, // price
        Decimal::from_str("10.0")?,  // size
        Side::Buy,
    );

    let options = CreateOrderOptions::default()
        .tick_size(Decimal::from_str("0.01")?)
        .neg_risk(false);

    trading_client.create_and_post_order(
        &order_args,
        None,        // expiration (defaults to 0 = no expiration)
        None,        // extras (defaults to ExtraOrderArgs::default())
        options,
        OrderType::Gtc,
    ).await?;

    println!("Order posted successfully!");

    Ok(())
}
```

### Client Types

| Client                | Purpose                                     | Authentication            |
| --------------------- | ------------------------------------------- | ------------------------- |
| `PublicClient`        | Market data queries                         | None                      |
| `AuthenticatedClient` | API key management, account operations      | L1 (EIP-712) or L2 (HMAC) |
| `TradingClient`       | Order creation, cancellation, trade queries | L2 (HMAC)                 |
| `DataClient`          | Position and portfolio data                 | None                      |

### Environment Setup

```bash
# Required for authenticated operations
export PRIVATE_KEY="0x..."

# Optional: specify custom API host
export POLYMARKET_HOST="https://clob.polymarket.com"
```

## Examples

See the [examples/](examples/) directory for complete working examples:

- [`public_data.rs`](examples/public_data.rs) - Public market data queries
- [`authenticated_trading.rs`](examples/authenticated_trading.rs) - Authenticated trading operations

Run an example:

```bash
cargo run --example public_data
PRIVATE_KEY="0x..." cargo run --example authenticated_trading
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Disclaimer

This is an unofficial library and is not affiliated with Polymarket. Use at your own risk. Always test with small amounts first on testnet before using real funds.
