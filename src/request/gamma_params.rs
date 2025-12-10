/// Query parameters for Gamma API market endpoints
#[derive(Debug, Clone, Default)]
pub struct GammaMarketParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub tag_id: Option<String>,
    pub order: Option<String>,
    pub ascending: Option<bool>,
}

impl GammaMarketParams {
    /// Create a new instance with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results to return
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the pagination offset
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Filter for active markets
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Filter for closed markets
    pub fn with_closed(mut self, closed: bool) -> Self {
        self.closed = Some(closed);
        self
    }

    /// Filter for archived markets
    pub fn with_archived(mut self, archived: bool) -> Self {
        self.archived = Some(archived);
        self
    }

    /// Filter by tag ID
    pub fn with_tag_id(mut self, tag_id: impl Into<String>) -> Self {
        self.tag_id = Some(tag_id.into());
        self
    }

    /// Set the ordering field
    pub fn with_order(mut self, order: impl Into<String>, ascending: bool) -> Self {
        self.order = Some(order.into());
        self.ascending = Some(ascending);
        self
    }

    /// Convert parameters to query string
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        if let Some(active) = self.active {
            params.push(format!("active={}", active));
        }
        if let Some(closed) = self.closed {
            params.push(format!("closed={}", closed));
        }
        if let Some(archived) = self.archived {
            params.push(format!("archived={}", archived));
        }
        if let Some(ref tag_id) = self.tag_id {
            params.push(format!("tag_id={}", tag_id));
        }
        if let Some(ref order) = self.order {
            params.push(format!("order={}", order));
        }
        if let Some(ascending) = self.ascending {
            params.push(format!("ascending={}", ascending));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_params() {
        let params = GammaMarketParams::new();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_basic_query_string() {
        let params = GammaMarketParams::new()
            .with_limit(10)
            .with_offset(20);

        let query = params.to_query_string();
        assert!(query.contains("limit=10"));
        assert!(query.contains("offset=20"));
        assert!(query.starts_with("?"));
    }

    #[test]
    fn test_active_filter() {
        let params = GammaMarketParams::new().with_active(true);

        let query = params.to_query_string();
        assert!(query.contains("active=true"));
    }

    #[test]
    fn test_ordering() {
        let params = GammaMarketParams::new()
            .with_order("volume", false);

        let query = params.to_query_string();
        assert!(query.contains("order=volume"));
        assert!(query.contains("ascending=false"));
    }

    #[test]
    fn test_combined_params() {
        let params = GammaMarketParams::new()
            .with_limit(5)
            .with_active(true)
            .with_closed(false)
            .with_tag_id("politics");

        let query = params.to_query_string();
        assert!(query.contains("limit=5"));
        assert!(query.contains("active=true"));
        assert!(query.contains("closed=false"));
        assert!(query.contains("tag_id=politics"));
    }
}
