use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::auth::KalshiAuth;
use crate::error::Error;
use crate::models::*;

const PROD_BASE: &str = "https://trading-api.kalshi.com/trade-api/v2";
const DEMO_BASE: &str = "https://demo-trading-api.kalshi.com/trade-api/v2";

/// The Kalshi API client.
#[derive(Debug, Clone)]
pub struct Kalshi {
    auth: KalshiAuth,
    http: Client,
    base_url: String,
}

impl Kalshi {
    /// Create a new client for the production Kalshi API.
    pub fn new(auth: KalshiAuth) -> Self {
        Self {
            auth,
            http: Client::new(),
            base_url: PROD_BASE.to_string(),
        }
    }

    /// Create a new client for the Kalshi demo environment.
    pub fn demo(auth: KalshiAuth) -> Self {
        Self {
            auth,
            http: Client::new(),
            base_url: DEMO_BASE.to_string(),
        }
    }

    /// Create a client with a custom base URL.
    pub fn with_base_url(auth: KalshiAuth, base_url: impl Into<String>) -> Self {
        Self {
            auth,
            http: Client::new(),
            base_url: base_url.into(),
        }
    }

    // ── Internal request helpers ──────────────────────────────────────

    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<T, Error> {
        let url = format!("{}{}", self.base_url, path);
        let headers = self.auth.sign(method.as_str(), path)?;

        debug!(method = %method, path, "kalshi request");

        let mut req = self
            .http
            .request(method, &url)
            .header("KALSHI-ACCESS-KEY", &headers.key)
            .header("KALSHI-ACCESS-SIGNATURE", &headers.signature)
            .header("KALSHI-ACCESS-TIMESTAMP", &headers.timestamp)
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        self.request::<T>(Method::GET, path, None::<&()>).await
    }

    async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, Error> {
        self.request(Method::POST, path, Some(body)).await
    }

    async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        self.request::<T>(Method::DELETE, path, None::<&()>).await
    }

    async fn handle_response<T: DeserializeOwned>(&self, resp: Response) -> Result<T, Error> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: body,
            });
        }
        let body = resp.text().await?;
        Ok(serde_json::from_str(&body)?)
    }

    // ── Exchange ──────────────────────────────────────────────────────

    /// Get exchange status.
    pub async fn get_exchange_status(&self) -> Result<serde_json::Value, Error> {
        self.get("/exchange/status").await
    }

    /// Get exchange schedule.
    pub async fn get_exchange_schedule(&self) -> Result<serde_json::Value, Error> {
        self.get("/exchange/schedule").await
    }

    // ── Events ───────────────────────────────────────────────────────

    /// Get a single event by ticker.
    pub async fn get_event(&self, event_ticker: &str) -> Result<EventResponse, Error> {
        self.get(&format!("/events/{event_ticker}")).await
    }

    /// Get events with optional cursor for pagination.
    pub async fn get_events(
        &self,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<EventsResponse, Error> {
        let mut path = "/events".to_string();
        let mut params = vec![];
        if let Some(c) = cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if !params.is_empty() {
            path = format!("{path}?{}", params.join("&"));
        }
        self.get(&path).await
    }

    // ── Markets ──────────────────────────────────────────────────────

    /// Start building a markets query.
    pub fn get_markets(&self) -> GetMarketsBuilder<'_> {
        GetMarketsBuilder::new(self)
    }

    /// Get a single market by ticker.
    pub async fn get_market(&self, ticker: &str) -> Result<MarketResponse, Error> {
        self.get(&format!("/markets/{ticker}")).await
    }

    /// Get market orderbook.
    pub async fn get_orderbook(
        &self,
        ticker: &str,
        depth: Option<u32>,
    ) -> Result<OrderbookResponse, Error> {
        let mut path = format!("/markets/{ticker}/orderbook");
        if let Some(d) = depth {
            path = format!("{path}?depth={d}");
        }
        self.get(&path).await
    }

    /// Get market candlesticks.
    pub async fn get_candlesticks(
        &self,
        ticker: &str,
        period_interval: u32,
    ) -> Result<CandlestickResponse, Error> {
        self.get(&format!(
            "/markets/{ticker}/candlesticks?period_interval={period_interval}"
        ))
        .await
    }

    /// Get market trades.
    pub async fn get_trades(
        &self,
        ticker: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<TradesResponse, Error> {
        let mut params = vec![];
        if let Some(t) = ticker {
            params.push(format!("ticker={t}"));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        let path = if params.is_empty() {
            "/markets/trades".to_string()
        } else {
            format!("/markets/trades?{}", params.join("&"))
        };
        self.get(&path).await
    }

    /// Get series list.
    pub async fn get_series_list(
        &self,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<SeriesListResponse, Error> {
        let mut params = vec![];
        if let Some(c) = cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        let path = if params.is_empty() {
            "/series".to_string()
        } else {
            format!("/series?{}", params.join("&"))
        };
        self.get(&path).await
    }

    // ── Portfolio ────────────────────────────────────────────────────

    /// Get account balance (in cents).
    pub async fn get_balance(&self) -> Result<BalanceResponse, Error> {
        self.get("/portfolio/balance").await
    }

    /// Get portfolio positions.
    pub async fn get_positions(
        &self,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<PositionsResponse, Error> {
        let mut params = vec![];
        if let Some(c) = cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        let path = if params.is_empty() {
            "/portfolio/positions".to_string()
        } else {
            format!("/portfolio/positions?{}", params.join("&"))
        };
        self.get(&path).await
    }

    /// Get portfolio fills (matched trades).
    pub async fn get_fills(
        &self,
        ticker: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<FillsResponse, Error> {
        let mut params = vec![];
        if let Some(t) = ticker {
            params.push(format!("ticker={t}"));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        let path = if params.is_empty() {
            "/portfolio/fills".to_string()
        } else {
            format!("/portfolio/fills?{}", params.join("&"))
        };
        self.get(&path).await
    }

    // ── Orders ───────────────────────────────────────────────────────

    /// Create a new order.
    pub async fn create_order(
        &self,
        order: &CreateOrderRequest,
    ) -> Result<CreateOrderResponse, Error> {
        self.post("/portfolio/orders", order).await
    }

    /// Get a single order by ID.
    pub async fn get_order(&self, order_id: &str) -> Result<Order, Error> {
        #[derive(serde::Deserialize)]
        struct Wrapper {
            order: Order,
        }
        let w: Wrapper = self.get(&format!("/portfolio/orders/{order_id}")).await?;
        Ok(w.order)
    }

    /// Cancel an order by ID.
    pub async fn cancel_order(&self, order_id: &str) -> Result<serde_json::Value, Error> {
        self.delete(&format!("/portfolio/orders/{order_id}")).await
    }

    /// Get all resting orders.
    pub async fn get_orders(
        &self,
        ticker: Option<&str>,
        status: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<OrdersResponse, Error> {
        let mut params = vec![];
        if let Some(t) = ticker {
            params.push(format!("ticker={t}"));
        }
        if let Some(s) = status {
            params.push(format!("status={s}"));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        let path = if params.is_empty() {
            "/portfolio/orders".to_string()
        } else {
            format!("/portfolio/orders?{}", params.join("&"))
        };
        self.get(&path).await
    }
}

// ── Builder for get_markets ──────────────────────────────────────────

/// Builder for the `GET /markets` endpoint with filters.
pub struct GetMarketsBuilder<'a> {
    client: &'a Kalshi,
    cursor: Option<String>,
    limit: Option<u32>,
    event_ticker: Option<String>,
    series_ticker: Option<String>,
    status: Option<String>,
    ticker: Option<String>,
}

impl<'a> GetMarketsBuilder<'a> {
    fn new(client: &'a Kalshi) -> Self {
        Self {
            client,
            cursor: None,
            limit: None,
            event_ticker: None,
            series_ticker: None,
            status: None,
            ticker: None,
        }
    }

    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    pub fn series_ticker(mut self, series_ticker: impl Into<String>) -> Self {
        self.series_ticker = Some(series_ticker.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Execute the query.
    pub async fn send(self) -> Result<MarketsResponse, Error> {
        let mut params = vec![];
        if let Some(c) = &self.cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(l) = self.limit {
            params.push(format!("limit={l}"));
        }
        if let Some(e) = &self.event_ticker {
            params.push(format!("event_ticker={e}"));
        }
        if let Some(s) = &self.series_ticker {
            params.push(format!("series_ticker={s}"));
        }
        if let Some(st) = &self.status {
            params.push(format!("status={st}"));
        }
        if let Some(t) = &self.ticker {
            params.push(format!("ticker={t}"));
        }
        let path = if params.is_empty() {
            "/markets".to_string()
        } else {
            format!("/markets?{}", params.join("&"))
        };
        self.client.get(&path).await
    }
}
