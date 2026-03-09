use serde::Deserialize;

/// A public trade on a market.
#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub trade_id: Option<String>,
    pub ticker: String,
    pub count: Option<i64>,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub taker_side: Option<String>,
    pub created_time: Option<String>,
}

/// Paginated trades response.
#[derive(Debug, Clone, Deserialize)]
pub struct TradesResponse {
    pub trades: Vec<Trade>,
    pub cursor: Option<String>,
}
