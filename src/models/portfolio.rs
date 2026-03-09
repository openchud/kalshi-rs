use serde::Deserialize;

/// Account balance response.
#[derive(Debug, Clone, Deserialize)]
pub struct BalanceResponse {
    /// Balance in cents.
    pub balance: f64,
}

/// A position in a market.
#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    pub ticker: String,
    pub event_ticker: Option<String>,
    pub market_result: Option<String>,
    pub yes_count: Option<i64>,
    pub no_count: Option<i64>,
    pub resting_orders_count: Option<i64>,
    pub total_cost: Option<i64>,
    pub fees_paid: Option<i64>,
    pub realized_pnl: Option<i64>,
}

/// Paginated positions response.
#[derive(Debug, Clone, Deserialize)]
pub struct PositionsResponse {
    pub market_positions: Vec<Position>,
    pub cursor: Option<String>,
}

/// A fill (matched trade) in portfolio.
#[derive(Debug, Clone, Deserialize)]
pub struct Fill {
    pub trade_id: Option<String>,
    pub ticker: String,
    pub order_id: Option<String>,
    pub side: Option<String>,
    pub action: Option<String>,
    pub count: Option<i64>,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub created_time: Option<String>,
    pub is_taker: Option<bool>,
}

/// Paginated fills response.
#[derive(Debug, Clone, Deserialize)]
pub struct FillsResponse {
    pub fills: Vec<Fill>,
    pub cursor: Option<String>,
}
