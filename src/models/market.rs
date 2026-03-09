use serde::{Deserialize, Serialize};

/// A Kalshi market (binary outcome contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub ticker: String,
    pub event_ticker: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub status: MarketStatus,
    pub yes_ask: Option<i64>,
    pub yes_bid: Option<i64>,
    pub no_ask: Option<i64>,
    pub no_bid: Option<i64>,
    pub last_price: Option<i64>,
    pub open_interest: Option<i64>,
    pub volume: Option<i64>,
    pub volume_24h: Option<i64>,
    pub close_time: Option<String>,
    pub result: Option<MarketResult>,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub rules_primary: Option<String>,
    pub rules_secondary: Option<String>,
    pub settlement_timer_seconds: Option<i64>,
    pub open_time: Option<String>,
    pub expiration_time: Option<String>,
    pub expected_expiration_time: Option<String>,
    pub settlement_value: Option<i64>,
    pub fee_waiver_expiration_time: Option<String>,
    pub series_ticker: Option<String>,
}

/// Market status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    Unopened,
    Open,
    Closed,
    Settled,
    #[serde(other)]
    Unknown,
}

/// Market result after settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketResult {
    Yes,
    No,
    #[serde(other)]
    Unknown,
}

/// Paginated response for markets.
#[derive(Debug, Clone, Deserialize)]
pub struct MarketsResponse {
    pub markets: Vec<Market>,
    pub cursor: Option<String>,
}

/// A single market response.
#[derive(Debug, Clone, Deserialize)]
pub struct MarketResponse {
    pub market: Market,
}

/// Orderbook for a market.
#[derive(Debug, Clone, Deserialize)]
pub struct Orderbook {
    pub yes: Vec<OrderbookLevel>,
    pub no: Vec<OrderbookLevel>,
}

/// A single price level in the orderbook.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookLevel {
    pub price: i64,
    pub quantity: i64,
}

/// Orderbook response wrapper.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookResponse {
    pub orderbook: Orderbook,
}

/// Candlestick data point.
#[derive(Debug, Clone, Deserialize)]
pub struct Candlestick {
    pub open: Option<i64>,
    pub high: Option<i64>,
    pub low: Option<i64>,
    pub close: Option<i64>,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
    #[serde(rename = "ts")]
    pub timestamp: Option<i64>,
    pub end_period_ts: Option<String>,
    pub start_period_ts: Option<String>,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub yes_ask: Option<i64>,
    pub yes_bid: Option<i64>,
}

/// Candlestick response.
#[derive(Debug, Clone, Deserialize)]
pub struct CandlestickResponse {
    pub candlesticks: Vec<Candlestick>,
}

/// Series information.
#[derive(Debug, Clone, Deserialize)]
pub struct Series {
    pub ticker: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub frequency: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Series list response.
#[derive(Debug, Clone, Deserialize)]
pub struct SeriesListResponse {
    pub series: Vec<Series>,
    pub cursor: Option<String>,
}
