use crate::error::Result;
use crate::http::HttpClient;
use crate::request::GammaMarketParams;
use crate::types::{GammaCategory, GammaEvent, GammaMarket, GammaSeries, GammaTag};

/// Client for Gamma API - Market discovery and metadata
///
/// This client provides access to Polymarket's Gamma API for market discovery
/// and metadata. The Gamma API indexes on-chain market data and provides rich
/// metadata including events, categories, tags, and volume metrics.
///
/// All endpoints are public and do not require authentication.
///
/// # Example
///
/// ```no_run
/// use polymarket_rs::client::GammaClient;
/// use polymarket_rs::request::GammaMarketParams;
///
/// #[tokio::main]
/// async fn main() -> polymarket_rs::Result<()> {
///     let client = GammaClient::new("https://gamma-api.polymarket.com");
///
///     // Get active markets
///     let params = GammaMarketParams::new()
///         .with_active(true)
///         .with_limit(10);
///
///     let markets = client.get_markets(Some(params)).await?;
///     println!("Found {} markets", markets.len());
///
///     Ok(())
/// }
/// ```
pub struct GammaClient {
    http_client: HttpClient,
}

impl GammaClient {
    /// Create a new GammaClient
    ///
    /// # Arguments
    /// * `host` - The base URL for the Gamma API (e.g., "https://gamma-api.polymarket.com")
    ///
    /// # Example
    /// ```
    /// use polymarket_rs::client::GammaClient;
    ///
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// ```
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            http_client: HttpClient::new(host),
        }
    }

    /// Get markets with optional filtering and pagination
    ///
    /// # Arguments
    /// * `params` - Optional query parameters for filtering and pagination
    ///
    /// # Returns
    /// A paginated response containing market data with rich metadata including
    /// events, categories, tags, volume breakdowns, and liquidity information.
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    /// use polymarket_rs::request::GammaMarketParams;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let params = GammaMarketParams::new()
    ///     .with_active(true)
    ///     .with_limit(10)
    ///     .with_offset(0);
    ///
    /// let markets = client.get_markets(Some(params)).await?;
    /// for market in markets {
    ///     if let Some(q) = &market.question {
    ///         println!("{}: {}", market.id, q);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_markets(
        &self,
        params: Option<GammaMarketParams>,
    ) -> Result<Vec<GammaMarket>> {
        let mut path = "/markets".to_string();
        if let Some(p) = params {
            path.push_str(&p.to_query_string());
        }
        self.http_client.get(&path, None).await
    }

    /// Get a specific market by condition ID
    ///
    /// # Arguments
    /// * `condition_id` - The condition ID of the market to retrieve
    ///
    /// # Returns
    /// A single market with full metadata
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let market = client.get_market("0x123...").await?;
    /// if let Some(q) = &market.question {
    ///     println!("Market: {}", q);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_market(&self, condition_id: &str) -> Result<GammaMarket> {
        let path = format!("/markets/{}", condition_id);
        self.http_client.get(&path, None).await
    }

    /// Get all available tags
    ///
    /// Tags are used for categorizing and filtering markets. This endpoint returns
    /// all tags available in the Gamma API.
    ///
    /// # Returns
    /// A list of all tags with their IDs, labels, and slugs
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let tags = client.get_tags().await?;
    /// for tag in tags {
    ///     println!("{}: {}", tag.slug, tag.label);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tags(&self) -> Result<Vec<GammaTag>> {
        self.http_client.get("/tags", None).await
    }

    /// Get all available categories
    ///
    /// Categories are high-level groupings for markets. This endpoint returns
    /// all categories available in the Gamma API.
    ///
    /// # Returns
    /// A list of all categories with their IDs, names, and slugs
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let categories = client.get_categories().await?;
    /// for category in categories {
    ///     println!("{}: {}", category.slug, category.label);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_categories(&self) -> Result<Vec<GammaCategory>> {
        self.http_client.get("/categories", None).await
    }

    /// Get a specific market by its ID
    ///
    /// # Arguments
    /// * `id` - The numeric ID of the market to retrieve
    ///
    /// # Returns
    /// A single market with full metadata
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let market = client.get_market_by_id("646091").await?;
    /// println!("Market: {:?}", market.question);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_market_by_id(&self, id: &str) -> Result<GammaMarket> {
        let path = format!("/markets/{}", id);
        self.http_client.get(&path, None).await
    }

    /// Get all events
    ///
    /// Events are collections of related markets. This endpoint returns
    /// all events available in the Gamma API.
    ///
    /// # Returns
    /// A list of all events with their metadata
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let events = client.get_events().await?;
    /// println!("Found {} events", events.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_events(&self) -> Result<Vec<GammaEvent>> {
        self.http_client.get("/events", None).await
    }

    /// Get a specific event by its ID
    ///
    /// # Arguments
    /// * `id` - The numeric ID of the event to retrieve
    ///
    /// # Returns
    /// A single event with full metadata
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let event = client.get_event_by_id("63806").await?;
    /// println!("Event: {:?}", event.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_event_by_id(&self, id: &str) -> Result<GammaEvent> {
        let path = format!("/events/{}", id);
        self.http_client.get(&path, None).await
    }

    /// Get all series
    ///
    /// Series are groupings of related events and markets. This endpoint returns
    /// all series available in the Gamma API.
    ///
    /// # Returns
    /// A list of all series with their metadata and nested events
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let series = client.get_series().await?;
    /// println!("Found {} series", series.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_series(&self) -> Result<Vec<GammaSeries>> {
        self.http_client.get("/series", None).await
    }

    /// Get a specific series by its ID
    ///
    /// # Arguments
    /// * `id` - The numeric ID of the series to retrieve
    ///
    /// # Returns
    /// A single series with full metadata and nested events
    ///
    /// # Example
    /// ```no_run
    /// use polymarket_rs::client::GammaClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> polymarket_rs::Result<()> {
    /// let client = GammaClient::new("https://gamma-api.polymarket.com");
    /// let series = client.get_series_by_id("10192").await?;
    /// println!("Series: {:?}", series.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_series_by_id(&self, id: &str) -> Result<GammaSeries> {
        let path = format!("/series/{}", id);
        self.http_client.get(&path, None).await
    }
}
