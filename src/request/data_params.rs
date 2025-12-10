/// Sort direction for activity queries
#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn as_str(&self) -> &str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

/// Sort field for activity queries
#[derive(Debug, Clone)]
pub enum ActivitySortBy {
    Timestamp,
}

impl ActivitySortBy {
    pub fn as_str(&self) -> &str {
        match self {
            ActivitySortBy::Timestamp => "TIMESTAMP",
        }
    }
}

/// Query parameters for trade endpoints with offset/limit pagination
#[derive(Debug, Clone, Default)]
pub struct TradeQueryParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub taker_only: Option<bool>,
}

impl TradeQueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_taker_only(mut self, taker_only: bool) -> Self {
        self.taker_only = Some(taker_only);
        self
    }

    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        if let Some(taker_only) = self.taker_only {
            params.push(format!("takerOnly={}", taker_only));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("&{}", params.join("&"))
        }
    }
}

/// Query parameters for activity endpoints with offset/limit pagination and sorting
#[derive(Debug, Clone, Default)]
pub struct ActivityQueryParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<ActivitySortBy>,
    pub sort_direction: Option<SortDirection>,
}

impl ActivityQueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_sort_by(mut self, sort_by: ActivitySortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    pub fn with_sort_direction(mut self, sort_direction: SortDirection) -> Self {
        self.sort_direction = Some(sort_direction);
        self
    }

    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(ref sort_by) = self.sort_by {
            params.push(format!("sortBy={}", sort_by.as_str()));
        }
        if let Some(ref sort_direction) = self.sort_direction {
            params.push(format!("sortDirection={}", sort_direction.as_str()));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("&{}", params.join("&"))
        }
    }
}
