use crate::error::{Error, Result};
use crate::types::OrderSummary;
use crate::Side;
use rust_decimal::Decimal;

/// Calculate the weighted average price for a market order based on order book depth
///
/// This walks the order book until enough liquidity is found to match
/// the requested shares, calculating the volume-weighted average price.
///
/// # Arguments
/// * `positions` - The order book positions to walk through
/// * `shares_to_match` - The number of shares to match
///
/// # Returns
/// The weighted average price at which the market order can be filled, or an error if there's insufficient liquidity
///
/// # Example
/// ```no_run
/// use polymarket_rs::orders::calculate_market_price;
/// use polymarket_rs::types::OrderSummary;
/// use polymarket_rs::Side;
/// use rust_decimal::Decimal;
///
/// let positions = vec![
///     OrderSummary { price: Decimal::new(50, 2), size: Decimal::new(100, 0) },
///     OrderSummary { price: Decimal::new(51, 2), size: Decimal::new(200, 0) },
/// ];
/// let price = calculate_market_price(&positions, Decimal::new(150, 0), Side::Buy).unwrap();
/// ```
pub fn calculate_market_price(
    positions: &[OrderSummary],
    shares_to_match: Decimal,
    side: Side,
) -> Result<Decimal> {
    let mut remaining = shares_to_match;
    let mut total_cost = Decimal::ZERO;

    // If buying, walk the asks (lowest to highest)
    // If selling, walk the bids (highest to lowest)
    let positions = match side {
        Side::Buy => {
            let mut asks = positions.to_vec();
            asks.sort_by(|a, b| a.price.cmp(&b.price));
            asks
        }
        Side::Sell => {
            let mut bids = positions.to_vec();
            bids.sort_by(|a, b| b.price.cmp(&a.price));
            bids
        }
    };

    for p in positions {
        let filled = remaining.min(p.size);
        total_cost += filled * p.price;
        remaining -= filled;

        if remaining.is_zero() {
            return Ok(total_cost / shares_to_match); // weighted avg price
        }
    }

    Err(Error::InvalidOrder(format!(
        "Not enough liquidity to create market order with amount {}",
        shares_to_match
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn order(price: Decimal, size: Decimal) -> OrderSummary {
        OrderSummary { price, size }
    }

    #[test]
    fn test_weighted_avg_price_buy() {
        // 10 shares @ 0.50, 20 shares @ 0.55
        let positions = vec![order(dec!(0.50), dec!(10)), order(dec!(0.55), dec!(20))];

        // Buy 25 shares: (10*0.50 + 15*0.55) / 25 = 13.25 / 25 = 0.53
        let price = calculate_market_price(&positions, dec!(25), Side::Buy).unwrap();
        assert_eq!(price, dec!(0.53));
    }

    #[test]
    fn test_weighted_avg_price_sell() {
        // Sells walk from highest to lowest
        let positions = vec![order(dec!(0.50), dec!(10)), order(dec!(0.55), dec!(20))];

        // Sell 25 shares: (20*0.55 + 5*0.50) / 25 = 13.50 / 25 = 0.54
        let price = calculate_market_price(&positions, dec!(25), Side::Sell).unwrap();
        assert_eq!(price, dec!(0.54));
    }

    #[test]
    fn test_single_tick() {
        let positions = vec![order(dec!(0.50), dec!(100))];
        let price = calculate_market_price(&positions, dec!(50), Side::Buy).unwrap();
        assert_eq!(price, dec!(0.50));
    }

    #[test]
    fn test_insufficient_liquidity() {
        let positions = vec![order(dec!(0.50), dec!(10))];
        let result = calculate_market_price(&positions, dec!(20), Side::Buy);
        assert!(result.is_err());
    }
}
