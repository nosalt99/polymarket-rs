use polymarket_rs::client::GammaClient;
use polymarket_rs::request::GammaMarketParams;
use polymarket_rs::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Polymarket Gamma API Client Test ===\n");

    let client = GammaClient::new("https://gamma-api.polymarket.com");

    println!("1. Fetching active markets (limit: 5)...");
    let params = GammaMarketParams::new()
        .with_active(true)
        .with_closed(false)
        .with_limit(5);

    match client.get_markets(Some(params)).await {
        Ok(markets) => {
            if let Some(market) = markets.first() {
                println!("   Sample Market:");
                println!("   - ID: {}", market.id);
                println!("   - Question: {}", market.question);
                println!("   - Condition ID: {}", market.condition_id);
                println!("   - Active: {}", market.active);
                println!("   - Closed: {}", market.closed);
                if let Some(vol) = &market.volume {
                    println!("   - Volume: {}", vol);
                }
                println!("   - Events: {}", market.events.len());
            }
        }
        Err(e) => {
            println!("Get markets error: {}", e);
        }
    }

    // Test 2: Get tags
    println!("\n2. Fetching available tags...");
    match client.get_tags().await {
        Ok(tags) => {
            println!("   Sample tags:");
            for tag in tags.iter().take(5) {
                println!("     - {} ({})", tag.label, tag.slug);
            }
        }
        Err(e) => {
            println!("Get tags error: {}", e);
        }
    }

    // Test 3: Get categories
    println!("\n3. Fetching available categories...");
    match client.get_categories().await {
        Ok(categories) => {
            println!("   Retrieved {} categories", categories.len());
        }
        Err(e) => {
            println!("Get categories error: {}", e);
        }
    }

    // Test 4: Get market by ID
    println!("\n4. Fetching specific market by ID (646091)...");
    match client.get_market_by_id("646091").await {
        Ok(market) => {
            println!("   - Question: {}", market.question);
            println!("   - Active: {}", market.active);
        }
        Err(e) => {
            println!("Get market by ID error: {}", e);
        }
    }

    // Test 5: Get events
    println!("\n5. Fetching all events...");
    match client.get_events().await {
        Ok(events) => {
            if let Some(event) = events.first() {
                println!("   Sample event: {}", event.title);
            }
        }
        Err(e) => {
            println!("Get events error: {}", e);
        }
    }

    // Test 6: Get event by ID
    println!("\n6. Fetching specific event by ID (63806)...");
    match client.get_event_by_id("63806").await {
        Ok(event) => {
            println!("   Sample event: {}", event.title);
            println!("   - Active: {}", event.active);
            println!("   - Start: {:?}", event.start_time);
            println!("   - End: {:?}", event.end_date);
        }
        Err(e) => {
            println!("Get event by ID error: {}", e);
        }
    }

    // Test 7: Get series
    println!("\n7. Fetching all series...");
    match client.get_series().await {
        Ok(series) => {
            if let Some(s) = series.first() {
                println!("   Sample series: {:?}", s.title);
            }
        }
        Err(e) => {
            println!("Get series error: {}", e);
        }
    }

    // Test 8: Get series by ID
    println!("\n8. Fetching specific series by ID (10192)...");
    match client.get_series_by_id("10192").await {
        Ok(series) => {
            println!("   - Title: {:?}", series.title);
            println!("   - Events: {}", series.events.len());
        }
        Err(e) => {
            println!("Get series by ID error: {}", e);
        }
    }

    println!("\n=== All tests completed! ===");
    Ok(())
}
