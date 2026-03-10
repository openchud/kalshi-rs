//! WebSocket streaming for real-time Kalshi market data.
//!
//! # Example
//!
//! ```rust,no_run
//! use kalshi::{Kalshi, KalshiAuth};
//! use kalshi::ws::{KalshiStream, Channel};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), kalshi::Error> {
//!     let auth = KalshiAuth::from_key_file("key-id", "key.pem")?;
//!     let mut stream = KalshiStream::connect(Some(&auth)).await?;
//!
//!     // Subscribe to ticker updates for all markets
//!     stream.subscribe(&[Channel::Ticker], None).await?;
//!
//!     // Subscribe to orderbook for specific markets
//!     stream.subscribe(
//!         &[Channel::OrderbookDelta],
//!         Some(&["KXGDP-26APR30-T3.0"]),
//!     ).await?;
//!
//!     // Process events
//!     while let Some(event) = stream.next().await? {
//!         match event {
//!             kalshi::ws::Event::Ticker(t) => {
//!                 println!("{}: bid={:?} ask={:?}", t.market_ticker, t.yes_bid, t.yes_ask);
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

mod messages;
mod stream;

pub use messages::*;
pub use stream::KalshiStream;
