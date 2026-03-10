use serde::{Deserialize, Serialize};

/// WebSocket channel to subscribe to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Real-time ticker updates (public).
    Ticker,
    /// Trade executions (public).
    Trade,
    /// Orderbook delta updates (auth required).
    OrderbookDelta,
    /// Fill notifications (auth required).
    Fill,
    /// Market lifecycle events (public).
    MarketLifecycle,
}

impl Channel {
    /// Channel name as used in the Kalshi WebSocket protocol.
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Ticker => "ticker",
            Channel::Trade => "trade",
            Channel::OrderbookDelta => "orderbook_delta",
            Channel::Fill => "fill",
            Channel::MarketLifecycle => "market_lifecycle_v2",
        }
    }
}

/// A command sent to the WebSocket.
#[derive(Debug, Serialize)]
pub(crate) struct WsCommand {
    pub id: u64,
    pub cmd: String,
    pub params: WsParams,
}

/// Parameters for a WebSocket command.
#[derive(Debug, Serialize)]
pub(crate) struct WsParams {
    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
}

/// An event received from the WebSocket.
#[derive(Debug, Clone)]
pub enum Event {
    /// Ticker update for a market.
    Ticker(TickerEvent),
    /// A trade occurred.
    Trade(TradeEvent),
    /// Orderbook snapshot.
    OrderbookSnapshot(OrderbookSnapshotEvent),
    /// Orderbook delta (incremental update).
    OrderbookDelta(OrderbookDeltaEvent),
    /// A fill on your order.
    Fill(FillEvent),
    /// Subscription confirmed.
    Subscribed { id: u64, channel: String },
    /// Error from server.
    Error { code: i64, message: String },
    /// Unknown message type.
    Unknown(serde_json::Value),
}

/// Ticker update event.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerEvent {
    pub market_ticker: String,
    pub yes_bid: Option<i64>,
    pub yes_ask: Option<i64>,
    pub no_bid: Option<i64>,
    pub no_ask: Option<i64>,
    pub last_price: Option<i64>,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
}

/// Trade event.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeEvent {
    pub market_ticker: String,
    pub count: Option<i64>,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub taker_side: Option<String>,
    pub ts: Option<i64>,
}

/// Full orderbook snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookSnapshotEvent {
    pub market_ticker: String,
    pub yes: Option<Vec<OrderbookLevelEvent>>,
    pub no: Option<Vec<OrderbookLevelEvent>>,
}

/// Orderbook level in a snapshot or delta.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookLevelEvent {
    pub price: i64,
    pub delta: Option<i64>,
    pub quantity: Option<i64>,
}

/// Incremental orderbook update.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookDeltaEvent {
    pub market_ticker: String,
    pub price: Option<i64>,
    pub delta: Option<i64>,
    pub side: Option<String>,
}

/// Fill event (your order was matched).
#[derive(Debug, Clone, Deserialize)]
pub struct FillEvent {
    pub trade_id: Option<String>,
    pub order_id: Option<String>,
    pub market_ticker: Option<String>,
    pub count: Option<i64>,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub is_taker: Option<bool>,
    pub ts: Option<i64>,
}

/// Parse a raw WebSocket JSON message into an Event.
pub(crate) fn parse_event(raw: &str) -> Result<Event, serde_json::Error> {
    let v: serde_json::Value = serde_json::from_str(raw)?;

    let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match msg_type {
        "ticker" => {
            let msg = v.get("msg").cloned().unwrap_or_default();
            let ticker: TickerEvent = serde_json::from_value(msg)?;
            Ok(Event::Ticker(ticker))
        }
        "trade" => {
            let msg = v.get("msg").cloned().unwrap_or_default();
            let trade: TradeEvent = serde_json::from_value(msg)?;
            Ok(Event::Trade(trade))
        }
        "orderbook_snapshot" => {
            let msg = v.get("msg").cloned().unwrap_or_default();
            let snap: OrderbookSnapshotEvent = serde_json::from_value(msg)?;
            Ok(Event::OrderbookSnapshot(snap))
        }
        "orderbook_delta" => {
            let msg = v.get("msg").cloned().unwrap_or_default();
            let delta: OrderbookDeltaEvent = serde_json::from_value(msg)?;
            Ok(Event::OrderbookDelta(delta))
        }
        "fill" => {
            let msg = v.get("msg").cloned().unwrap_or_default();
            let fill: FillEvent = serde_json::from_value(msg)?;
            Ok(Event::Fill(fill))
        }
        "subscribed" => {
            let id = v.get("id").and_then(|i| i.as_u64()).unwrap_or(0);
            let channel = v
                .get("msg")
                .and_then(|m| m.get("channel"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            Ok(Event::Subscribed { id, channel })
        }
        "error" => {
            let code = v
                .get("msg")
                .and_then(|m| m.get("code"))
                .and_then(|c| c.as_i64())
                .unwrap_or(0);
            let message = v
                .get("msg")
                .and_then(|m| m.get("msg"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error")
                .to_string();
            Ok(Event::Error { code, message })
        }
        _ => Ok(Event::Unknown(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ticker() {
        let raw = r#"{"type":"ticker","msg":{"market_ticker":"KXTEST","yes_bid":45,"yes_ask":48}}"#;
        let event = parse_event(raw).unwrap();
        match event {
            Event::Ticker(t) => {
                assert_eq!(t.market_ticker, "KXTEST");
                assert_eq!(t.yes_bid, Some(45));
                assert_eq!(t.yes_ask, Some(48));
            }
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn parse_trade() {
        let raw = r#"{"type":"trade","msg":{"market_ticker":"KXTEST","count":10,"yes_price":60,"taker_side":"yes"}}"#;
        let event = parse_event(raw).unwrap();
        match event {
            Event::Trade(t) => {
                assert_eq!(t.market_ticker, "KXTEST");
                assert_eq!(t.count, Some(10));
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[test]
    fn parse_orderbook_snapshot() {
        let raw = r#"{"type":"orderbook_snapshot","msg":{"market_ticker":"KXTEST","yes":[{"price":45,"quantity":100}],"no":[]}}"#;
        let event = parse_event(raw).unwrap();
        match event {
            Event::OrderbookSnapshot(s) => {
                assert_eq!(s.market_ticker, "KXTEST");
                assert_eq!(s.yes.unwrap().len(), 1);
            }
            _ => panic!("Expected OrderbookSnapshot"),
        }
    }

    #[test]
    fn parse_error() {
        let raw = r#"{"id":1,"type":"error","msg":{"code":2,"msg":"Params required"}}"#;
        let event = parse_event(raw).unwrap();
        match event {
            Event::Error { code, message } => {
                assert_eq!(code, 2);
                assert_eq!(message, "Params required");
            }
            _ => panic!("Expected Error event"),
        }
    }

    #[test]
    fn parse_subscribed() {
        let raw = r#"{"id":1,"type":"subscribed","msg":{"channel":"ticker"}}"#;
        let event = parse_event(raw).unwrap();
        match event {
            Event::Subscribed { id, channel } => {
                assert_eq!(id, 1);
                assert_eq!(channel, "ticker");
            }
            _ => panic!("Expected Subscribed event"),
        }
    }

    #[test]
    fn parse_fill() {
        let raw = r#"{"type":"fill","msg":{"trade_id":"t-1","order_id":"o-1","market_ticker":"KXTEST","count":5,"is_taker":true}}"#;
        let event = parse_event(raw).unwrap();
        match event {
            Event::Fill(f) => {
                assert_eq!(f.market_ticker, Some("KXTEST".into()));
                assert_eq!(f.count, Some(5));
                assert_eq!(f.is_taker, Some(true));
            }
            _ => panic!("Expected Fill event"),
        }
    }

    #[test]
    fn parse_unknown() {
        let raw = r#"{"type":"future_type","msg":{"foo":"bar"}}"#;
        let event = parse_event(raw).unwrap();
        assert!(matches!(event, Event::Unknown(_)));
    }

    #[test]
    fn channel_names() {
        assert_eq!(Channel::Ticker.as_str(), "ticker");
        assert_eq!(Channel::OrderbookDelta.as_str(), "orderbook_delta");
        assert_eq!(Channel::Fill.as_str(), "fill");
        assert_eq!(Channel::MarketLifecycle.as_str(), "market_lifecycle_v2");
    }
}
