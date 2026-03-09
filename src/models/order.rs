use serde::{Deserialize, Serialize};

/// Side of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Yes,
    No,
}

/// Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
}

/// Order action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderAction {
    Buy,
    Sell,
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Resting,
    Pending,
    Executed,
    Canceled,
    #[serde(other)]
    Unknown,
}

/// Request to create a new order.
#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    pub ticker: String,
    pub action: OrderAction,
    pub side: Side,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub count: i64,
    /// Price in cents (1-99 for limit orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,
    /// Price in cents (1-99 for limit orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Buy order only: maximum cost willing to pay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<i64>,
    /// Sell order only: minimum revenue willing to accept.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_position_floor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,
}

/// An order on Kalshi.
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub ticker: String,
    pub action: OrderAction,
    pub side: Side,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
    pub expiration_time: Option<String>,
    pub close_cancel_count: Option<i64>,
    pub place_count: Option<i64>,
    pub decrease_count: Option<i64>,
    pub maker_fill_count: Option<i64>,
    pub taker_fill_count: Option<i64>,
    pub remaining_count: Option<i64>,
    pub queue_position: Option<i64>,
    pub user_id: Option<String>,
    pub last_update_time: Option<String>,
    pub taker_fees: Option<i64>,
    pub client_order_id: Option<String>,
}

/// Create order response.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderResponse {
    pub order: Order,
}

/// Paginated orders response.
#[derive(Debug, Clone, Deserialize)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
    pub cursor: Option<String>,
}
