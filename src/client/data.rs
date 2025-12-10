use crate::error::Result;
use crate::http::HttpClient;
use crate::request::{ActivityQueryParams, TradeQueryParams};
use crate::types::{Activity, ClosedPosition, Position, PositionValue, Trade};

/// Client for accessing position and portfolio data
///
/// This client provides access to user positions and portfolio values.
/// It does not require authentication.
pub struct DataClient {
    http_client: HttpClient,
}

impl DataClient {
    /// Create a new DataClient
    ///
    /// # Arguments
    /// * `host` - The base URL for the data API (typically different from main CLOB API)
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            http_client: HttpClient::new(host),
        }
    }

    /// Get all positions for a user
    ///
    /// # Arguments
    /// * `user` - The user's wallet address
    ///
    /// # Returns
    /// A list of positions owned by the user
    pub async fn get_positions(&self, user: &str) -> Result<Vec<Position>> {
        let path = format!("/positions?user={}", user);
        self.http_client.get(&path, None).await
    }

    /// Get the total value of positions for a user
    ///
    /// # Arguments
    /// * `user` - The user's wallet address
    ///
    /// # Returns
    /// A list of position values for the user
    pub async fn get_positions_value(&self, user: &str) -> Result<Vec<PositionValue>> {
        let path = format!("/value?user={}", user);
        self.http_client.get(&path, None).await
    }

    /// Get recent trades
    ///
    /// # Arguments
    /// * `user` - User wallet address to filter trades
    /// * `params` - Optional query parameters (limit, offset, taker_only)
    ///
    /// # Returns
    /// A list of recent trades
    pub async fn get_trades(
        &self,
        user: &str,
        params: Option<TradeQueryParams>,
    ) -> Result<Vec<Trade>> {
        let mut path = format!("/trades?user={}", user);

        if let Some(params) = params {
            path.push_str(&params.to_query_string());
        }

        println!("{}", path);

        self.http_client.get(&path, None).await
    }

    /// Get recent activity
    ///
    /// # Arguments
    /// * `user` - User wallet address to filter activity
    /// * `params` - Optional query parameters (limit, offset, sort_by, sort_direction)
    ///
    /// # Returns
    /// A list of recent activity events
    pub async fn get_activity(
        &self,
        user: &str,
        params: Option<ActivityQueryParams>,
    ) -> Result<Vec<Activity>> {
        let mut path = format!("/activity?user={}", user);

        if let Some(params) = params {
            path.push_str(&params.to_query_string());
        }

        self.http_client.get(&path, None).await
    }

    /// Get closed positions
    ///
    /// # Arguments
    /// * `user` - User wallet address
    ///
    /// # Returns
    /// A list of closed positions for the user
    pub async fn get_closed_positions(&self, user: &str) -> Result<Vec<ClosedPosition>> {
        let path = format!("/closed-positions?user={}", user);
        self.http_client.get(&path, None).await
    }
}
