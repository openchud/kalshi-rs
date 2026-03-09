#[cfg(test)]
mod tests {
    use crate::models::*;

    #[test]
    fn deserialize_market() {
        let json = r#"{
            "ticker": "KXTEST-001",
            "event_ticker": "KXTEST",
            "title": "Will it rain?",
            "status": "open",
            "yes_bid": 45,
            "yes_ask": 48,
            "no_bid": 52,
            "no_ask": 55,
            "last_price": 46,
            "open_interest": 1000,
            "volume": 5000,
            "volume_24h": 200,
            "tags": ["weather"]
        }"#;
        let market: Market = serde_json::from_str(json).unwrap();
        assert_eq!(market.ticker, "KXTEST-001");
        assert_eq!(market.status, MarketStatus::Open);
        assert_eq!(market.yes_bid, Some(45));
        assert_eq!(market.tags, vec!["weather"]);
    }

    #[test]
    fn deserialize_markets_response() {
        let json = r#"{
            "markets": [
                {
                    "ticker": "KXTEST-001",
                    "event_ticker": "KXTEST",
                    "title": "Test market",
                    "status": "settled",
                    "result": "yes",
                    "tags": []
                }
            ],
            "cursor": "abc123"
        }"#;
        let resp: MarketsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.markets.len(), 1);
        assert_eq!(resp.markets[0].status, MarketStatus::Settled);
        assert_eq!(resp.markets[0].result, Some(MarketResult::Yes));
        assert_eq!(resp.cursor, Some("abc123".to_string()));
    }

    #[test]
    fn deserialize_orderbook() {
        let json = r#"{
            "orderbook": {
                "yes": [{"price": 45, "quantity": 100}, {"price": 44, "quantity": 200}],
                "no": [{"price": 55, "quantity": 150}]
            }
        }"#;
        let resp: OrderbookResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.orderbook.yes.len(), 2);
        assert_eq!(resp.orderbook.yes[0].price, 45);
        assert_eq!(resp.orderbook.no[0].quantity, 150);
    }

    #[test]
    fn deserialize_order() {
        let json = r#"{
            "order_id": "ord-123",
            "ticker": "KXTEST-001",
            "action": "buy",
            "side": "yes",
            "type": "limit",
            "status": "resting",
            "yes_price": 45,
            "place_count": 10,
            "remaining_count": 10
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, "ord-123");
        assert_eq!(order.action, OrderAction::Buy);
        assert_eq!(order.side, Side::Yes);
        assert_eq!(order.status, OrderStatus::Resting);
    }

    #[test]
    fn deserialize_position() {
        let json = r#"{
            "ticker": "KXTEST-001",
            "event_ticker": "KXTEST",
            "yes_count": 50,
            "no_count": 0,
            "total_cost": 2250,
            "fees_paid": 50
        }"#;
        let pos: Position = serde_json::from_str(json).unwrap();
        assert_eq!(pos.ticker, "KXTEST-001");
        assert_eq!(pos.yes_count, Some(50));
        assert_eq!(pos.total_cost, Some(2250));
    }

    #[test]
    fn deserialize_balance() {
        let json = r#"{"balance": 23983.0}"#;
        let bal: BalanceResponse = serde_json::from_str(json).unwrap();
        assert!((bal.balance - 23983.0).abs() < f64::EPSILON);
    }

    #[test]
    fn serialize_create_order() {
        let order = CreateOrderRequest {
            ticker: "KXTEST-001".into(),
            action: OrderAction::Buy,
            side: Side::No,
            order_type: OrderType::Limit,
            count: 10,
            yes_price: None,
            no_price: Some(72),
            client_order_id: None,
            buy_max_cost: None,
            sell_position_floor: None,
            expiration_ts: None,
        };
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["ticker"], "KXTEST-001");
        assert_eq!(json["action"], "buy");
        assert_eq!(json["side"], "no");
        assert_eq!(json["type"], "limit");
        assert_eq!(json["count"], 10);
        assert_eq!(json["no_price"], 72);
        // Optional None fields should be absent
        assert!(json.get("yes_price").is_none());
        assert!(json.get("client_order_id").is_none());
    }

    #[test]
    fn unknown_status_deserializes() {
        let json = r#"{
            "ticker": "KXTEST",
            "event_ticker": "KX",
            "title": "Test",
            "status": "some_future_status",
            "tags": []
        }"#;
        let market: Market = serde_json::from_str(json).unwrap();
        assert_eq!(market.status, MarketStatus::Unknown);
    }

    #[test]
    fn deserialize_event() {
        let json = r#"{
            "event_ticker": "KXGDP-26APR30",
            "title": "GDP Q1 2026",
            "category": "economics",
            "mutually_exclusive": true,
            "markets": ["KXGDP-26APR30-T1.0", "KXGDP-26APR30-T1.5"]
        }"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_ticker, "KXGDP-26APR30");
        assert_eq!(event.markets.len(), 2);
    }

    #[test]
    fn deserialize_fill() {
        let json = r#"{
            "trade_id": "t-001",
            "ticker": "KXTEST-001",
            "order_id": "ord-123",
            "side": "yes",
            "action": "buy",
            "count": 10,
            "yes_price": 45,
            "is_taker": true
        }"#;
        let fill: Fill = serde_json::from_str(json).unwrap();
        assert_eq!(fill.ticker, "KXTEST-001");
        assert_eq!(fill.count, Some(10));
        assert_eq!(fill.is_taker, Some(true));
    }

    #[test]
    fn deserialize_trade() {
        let json = r#"{
            "trade_id": "t-public-001",
            "ticker": "KXTEST-001",
            "count": 25,
            "yes_price": 60,
            "taker_side": "yes"
        }"#;
        let trade: Trade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.ticker, "KXTEST-001");
        assert_eq!(trade.yes_price, Some(60));
    }
}
