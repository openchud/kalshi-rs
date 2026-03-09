# kalshi-rs

Typed Rust SDK for the [Kalshi](https://kalshi.com) prediction market API.

## Features

- **Full API coverage**: Markets, events, orders, positions, orderbook, candlesticks, trades
- **RSA-PSS authentication**: Built-in request signing per Kalshi's API spec
- **Builder pattern**: Ergonomic query construction for filtered endpoints
- **Async/await**: Built on `reqwest` + `tokio`
- **Strong types**: Every response is deserialized into typed Rust structs
- **Demo mode**: First-class support for Kalshi's demo environment

## Quick Start

```rust
use kalshi::{Kalshi, KalshiAuth};

#[tokio::main]
async fn main() -> Result<(), kalshi::Error> {
    let auth = KalshiAuth::from_key_file("your-api-key-id", "path/to/private_key.pem")?;
    let client = Kalshi::new(auth);

    // Get balance
    let balance = client.get_balance().await?;
    println!("Balance: ${:.2}", balance.balance / 100.0);

    // List open markets
    let markets = client.get_markets()
        .status("open")
        .limit(10)
        .send()
        .await?;

    for m in &markets.markets {
        println!("{}: {} (yes_bid: {:?})", m.ticker, m.title, m.yes_bid);
    }

    // Get orderbook
    let book = client.get_orderbook("KXGDP-26APR30-T3.0", Some(5)).await?;
    println!("Yes bids: {:?}", book.orderbook.yes);

    // Place a limit order
    use kalshi::models::{CreateOrderRequest, OrderAction, OrderType, Side};
    let order = CreateOrderRequest {
        ticker: "KXGDP-26APR30-T3.0".into(),
        action: OrderAction::Buy,
        side: Side::No,
        order_type: OrderType::Limit,
        count: 10,
        no_price: Some(72),
        yes_price: None,
        client_order_id: None,
        buy_max_cost: None,
        sell_position_floor: None,
        expiration_ts: None,
    };
    let result = client.create_order(&order).await?;
    println!("Order placed: {}", result.order.order_id);

    Ok(())
}
```

## Demo Environment

```rust
let client = Kalshi::demo(auth); // Points to demo-trading-api.kalshi.com
```

## Authentication

Kalshi uses RSA-PSS signatures. Generate an API key from your [Kalshi account](https://kalshi.com/account/api-keys), download the private key PEM file, then:

```rust
let auth = KalshiAuth::from_key_file("KALSHI-API-KEY-ID", "/path/to/key.pem")?;
// or from a string:
let auth = KalshiAuth::from_key_pem("KALSHI-API-KEY-ID", &pem_string)?;
```

## License

MIT
