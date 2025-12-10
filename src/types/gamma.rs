use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Gamma API market with rich metadata
/// Note: Most fields are optional since the API has inconsistent data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GammaMarket {
    pub id: String,
    pub question: String,
    pub description: String,
    pub outcomes: Option<String>,       // JSON string
    pub outcome_prices: Option<String>, // JSON string
    pub clob_token_ids: Option<String>, // JSON string
    pub condition_id: String,

    // Status flags
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub restricted: bool,

    // Metadata
    pub slug: String,
    pub category: Option<String>,
    pub market_type: Option<String>,

    // Trading data as strings to avoid parsing issues
    pub volume: Option<String>,
    pub liquidity: Option<String>,
    pub volume_num: Option<f64>,
    pub liquidity_num: Option<f64>,
    pub volume24hr: Option<f64>,

    // Price data
    pub last_trade_price: Option<f64>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub spread: Option<f64>,
    // Nested data
    #[serde(default)]
    pub events: Vec<GammaSimplifiedEvent>,
}

/// Event associated with a market
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GammaSimplifiedEvent {
    pub id: String,
    pub ticker: String,
    pub slug: String,
    pub title: String,

    // Dates
    #[serde(
        default,
        deserialize_with = "super::serde_helpers::deserialize_optional_datetime"
    )]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(
        default,
        deserialize_with = "super::serde_helpers::deserialize_optional_datetime"
    )]
    pub start_time: Option<DateTime<Utc>>,

    // Status flags
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub new: bool,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub restricted: bool,

    // Order book settings
    #[serde(default)]
    pub enable_order_book: bool,

    // Risk settings
    #[serde(default)]
    pub neg_risk: bool,
    #[serde(default)]
    pub enable_neg_risk: bool,
    #[serde(default)]
    pub neg_risk_augmented: bool,

    // Tags
    #[serde(default)]
    pub tags: Vec<GammaTag>,
}

/// Event associated with a market
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GammaEvent {
    pub id: String,
    pub ticker: String,
    pub slug: String,
    pub title: String,

    // Dates
    #[serde(
        default,
        deserialize_with = "super::serde_helpers::deserialize_optional_datetime"
    )]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(
        default,
        deserialize_with = "super::serde_helpers::deserialize_optional_datetime"
    )]
    pub start_time: Option<DateTime<Utc>>,

    // Status flags
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub new: bool,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub restricted: bool,

    // Trading data
    pub volume: Option<f64>,
    pub liquidity: Option<f64>,
    pub open_interest: Option<f64>,
    pub competitive: Option<f64>,
    pub liquidity_clob: Option<f64>,

    // Order book settings
    #[serde(default)]
    pub enable_order_book: bool,

    // Risk settings
    #[serde(default)]
    pub neg_risk: bool,
    #[serde(default)]
    pub enable_neg_risk: bool,
    #[serde(default)]
    pub neg_risk_augmented: bool,

    // Tags
    #[serde(default)]
    pub tags: Vec<GammaTag>,

    // Additional flags
    #[serde(default)]
    pub cyom: bool,
    #[serde(default)]
    pub show_all_outcomes: bool,
    #[serde(default)]
    pub show_market_images: bool,
    #[serde(default)]
    pub automatically_active: bool,
    #[serde(default)]
    pub pending_deployment: bool,
    #[serde(default)]
    pub deploying: bool,

    // Series reference
    pub series_slug: Option<String>,

    // Additional metadata
    pub category: Option<String>,
    pub sort_by: Option<String>,

    // Additional volume metrics
    pub volume24hr: Option<f64>,
    pub volume1wk: Option<f64>,
    pub volume1mo: Option<f64>,
    pub volume1yr: Option<f64>,
    pub liquidity_amm: Option<f64>,

    pub markets: Vec<GammaMarket>,
}

/// Tag for market categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GammaTag {
    pub id: String,
    pub label: String,
    pub slug: String,
    #[serde(default)]
    pub force_show: bool,
    #[serde(default)]
    pub is_carousel: bool,
}

/// Category for market organization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GammaCategory {
    pub id: String,
    pub label: String, // Note: API uses "label", not "name"
    pub slug: String,
}

/// Series grouping multiple related events/markets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GammaSeries {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: String,
    pub title: Option<String>,
    pub series_type: Option<String>,
    pub recurrence: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub layout: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub new: bool,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    #[serde(default)]
    pub comments_enabled: bool,
    pub competitive: Option<String>,
    pub volume24hr: Option<f64>,
    pub comment_count: Option<i64>,

    #[serde(default)]
    pub events: Vec<GammaSimplifiedEvent>,
}
