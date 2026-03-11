//! # kalshi
//!
//! Typed Rust SDK for the [Kalshi](https://kalshi.com) prediction market API.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use kalshi::{Kalshi, KalshiAuth};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), kalshi::Error> {
//!     let auth = KalshiAuth::from_key_file("your-api-key-id", "path/to/private_key.pem")?;
//!     let client = Kalshi::new(auth);
//!
//!     // Get account balance
//!     let balance = client.get_balance().await?;
//!     println!("Balance: ${:.2}", balance.balance / 100.0);
//!
//!     // List markets
//!     let markets = client.get_markets().limit(10).send().await?;
//!     for market in &markets.markets {
//!         println!("{}: {}", market.ticker, market.title);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod ws;

pub use auth::KalshiAuth;
pub use client::Kalshi;
pub use error::Error;

#[cfg(test)]
mod tests;
