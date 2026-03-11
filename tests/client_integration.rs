//! Integration tests for the Kalshi HTTP client using wiremock.

use kalshi::auth::KalshiAuth;
use kalshi::client::Kalshi;
use kalshi::error::Error;
use kalshi::models::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_KEY_PEM: &str = include_str!("test_key.pem");

fn test_client(base_url: &str) -> Kalshi {
    let auth = KalshiAuth::from_key_pem("test-key-id", TEST_KEY_PEM).unwrap();
    Kalshi::with_base_url(auth, base_url)
}

// ── Markets ──────────────────────────────────────────────────────────

#[tokio::test]
async fn get_market_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/KXTEST-001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "market": {
                "ticker": "KXTEST-001",
                "event_ticker": "KXTEST",
                "title": "Will it rain?",
                "status": "open",
                "yes_bid": 45,
                "tags": []
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_market("KXTEST-001").await.unwrap();
    assert_eq!(resp.market.ticker, "KXTEST-001");
    assert_eq!(resp.market.status, MarketStatus::Open);
}

#[tokio::test]
async fn get_markets_builder() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("status", "open"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [{
                "ticker": "KXTEST-001",
                "event_ticker": "KXTEST",
                "title": "Test",
                "status": "open",
                "tags": []
            }],
            "cursor": "next-page"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_markets().status("open").limit(10).send().await.unwrap();
    assert_eq!(resp.markets.len(), 1);
    assert_eq!(resp.cursor, Some("next-page".to_string()));
}

#[tokio::test]
async fn get_markets_pagination() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("cursor", "page2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [],
            "cursor": null
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_markets().cursor("page2").send().await.unwrap();
    assert!(resp.markets.is_empty());
    assert!(resp.cursor.is_none());
}

// ── Orderbook ────────────────────────────────────────────────────────

#[tokio::test]
async fn get_orderbook_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/KXTEST-001/orderbook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orderbook": {
                "yes": [{"price": 45, "quantity": 100}],
                "no": [{"price": 55, "quantity": 200}]
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_orderbook("KXTEST-001", None).await.unwrap();
    assert_eq!(resp.orderbook.yes.len(), 1);
    assert_eq!(resp.orderbook.yes[0].price, 45);
}

#[tokio::test]
async fn get_orderbook_with_depth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/KXTEST-001/orderbook"))
        .and(query_param("depth", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orderbook": { "yes": [], "no": [] }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_orderbook("KXTEST-001", Some(5)).await.unwrap();
    assert!(resp.orderbook.yes.is_empty());
}

// ── Portfolio ────────────────────────────────────────────────────────

#[tokio::test]
async fn get_balance_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "balance": 50000.0
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_balance().await.unwrap();
    assert!((resp.balance - 50000.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn get_positions_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/positions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "market_positions": [{
                "ticker": "KXTEST-001",
                "yes_count": 50,
                "no_count": 0,
                "total_cost": 2250
            }],
            "cursor": null
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_positions(None, None).await.unwrap();
    assert_eq!(resp.market_positions.len(), 1);
    assert_eq!(resp.market_positions[0].yes_count, Some(50));
}

#[tokio::test]
async fn get_fills_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/fills"))
        .and(query_param("ticker", "KXTEST-001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "fills": [{
                "trade_id": "t-1",
                "ticker": "KXTEST-001",
                "count": 10,
                "is_taker": true
            }],
            "cursor": null
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_fills(Some("KXTEST-001"), None, None).await.unwrap();
    assert_eq!(resp.fills.len(), 1);
}

// ── Orders ───────────────────────────────────────────────────────────

#[tokio::test]
async fn create_order_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/portfolio/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "order": {
                "order_id": "ord-123",
                "ticker": "KXTEST-001",
                "action": "buy",
                "side": "yes",
                "type": "limit",
                "status": "resting",
                "yes_price": 45,
                "place_count": 10,
                "remaining_count": 10
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let order = CreateOrderRequest {
        ticker: "KXTEST-001".into(),
        action: OrderAction::Buy,
        side: Side::Yes,
        order_type: OrderType::Limit,
        count: 10,
        yes_price: Some(45),
        no_price: None,
        client_order_id: None,
        buy_max_cost: None,
        sell_position_floor: None,
        expiration_ts: None,
    };
    let resp = client.create_order(&order).await.unwrap();
    assert_eq!(resp.order.order_id, "ord-123");
    assert_eq!(resp.order.status, OrderStatus::Resting);
}

#[tokio::test]
async fn cancel_order_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/portfolio/orders/ord-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "order": {"order_id": "ord-123", "status": "canceled"}
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.cancel_order("ord-123").await.unwrap();
    assert!(resp.is_object());
}

#[tokio::test]
async fn get_order_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/orders/ord-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "order": {
                "order_id": "ord-123",
                "ticker": "KXTEST-001",
                "action": "buy",
                "side": "yes",
                "type": "limit",
                "status": "executed"
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let order = client.get_order("ord-123").await.unwrap();
    assert_eq!(order.order_id, "ord-123");
    assert_eq!(order.status, OrderStatus::Executed);
}

// ── Events ───────────────────────────────────────────────────────────

#[tokio::test]
async fn get_event_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/KXGDP"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "event": {
                "event_ticker": "KXGDP",
                "title": "GDP Q1",
                "markets": ["KXGDP-T1", "KXGDP-T2"]
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_event("KXGDP").await.unwrap();
    assert_eq!(resp.event.event_ticker, "KXGDP");
    assert_eq!(resp.event.markets.len(), 2);
}

// ── Trades ───────────────────────────────────────────────────────────

#[tokio::test]
async fn get_trades_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/trades"))
        .and(query_param("ticker", "KXTEST-001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "trades": [{
                "trade_id": "t-1",
                "ticker": "KXTEST-001",
                "count": 25,
                "yes_price": 60
            }],
            "cursor": null
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_trades(Some("KXTEST-001"), None, None).await.unwrap();
    assert_eq!(resp.trades.len(), 1);
}

// ── Candlesticks ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_candlesticks_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/KXTEST-001/candlesticks"))
        .and(query_param("period_interval", "60"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candlesticks": [{
                "open": 45,
                "high": 50,
                "low": 40,
                "close": 48,
                "volume": 1000
            }]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_candlesticks("KXTEST-001", 60).await.unwrap();
    assert_eq!(resp.candlesticks.len(), 1);
    assert_eq!(resp.candlesticks[0].high, Some(50));
}

// ── Series ───────────────────────────────────────────────────────────

#[tokio::test]
async fn get_series_list_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/series"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "series": [{"ticker": "KXGDP", "title": "GDP Series"}],
            "cursor": null
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_series_list(None, None).await.unwrap();
    assert_eq!(resp.series.len(), 1);
}

// ── Error handling ───────────────────────────────────────────────────

#[tokio::test]
async fn api_error_403() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/balance"))
        .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let err = client.get_balance().await.unwrap_err();
    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 403);
            assert_eq!(message, "Forbidden");
        }
        other => panic!("Expected Api error, got: {other}"),
    }
}

#[tokio::test]
async fn api_error_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/NONEXISTENT"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let err = client.get_market("NONEXISTENT").await.unwrap_err();
    match err {
        Error::Api { status, .. } => assert_eq!(status, 404),
        other => panic!("Expected Api error, got: {other}"),
    }
}

#[tokio::test]
async fn api_error_500() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/exchange/status"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let err = client.get_exchange_status().await.unwrap_err();
    match err {
        Error::Api { status, .. } => assert_eq!(status, 500),
        other => panic!("Expected Api error, got: {other}"),
    }
}

#[tokio::test]
async fn api_error_429_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Rate limit exceeded"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let err = client.get_markets().send().await.unwrap_err();
    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 429);
            assert!(message.contains("Rate limit"));
        }
        other => panic!("Expected Api error, got: {other}"),
    }
}

#[tokio::test]
async fn malformed_json_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let err = client.get_balance().await.unwrap_err();
    assert!(matches!(err, Error::Json(_)));
}
