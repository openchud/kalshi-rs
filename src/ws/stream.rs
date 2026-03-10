use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite};
use tracing::{debug, warn};

use crate::auth::KalshiAuth;
use crate::error::Error;
use crate::ws::messages::{Channel, Event, WsCommand, WsParams, parse_event};

const PROD_WS: &str = "wss://api.elections.kalshi.com/trade-api/ws/v2";
const DEMO_WS: &str = "wss://demo-api.kalshi.co/trade-api/ws/v2";

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// A WebSocket connection to the Kalshi streaming API.
pub struct KalshiStream {
    ws: WsStream,
    next_id: u64,
}

impl KalshiStream {
    /// Connect to the production WebSocket API.
    ///
    /// Pass `Some(&auth)` for authenticated channels (orderbook_delta, fill).
    /// Pass `None` for public-only channels (ticker, trade).
    pub async fn connect(auth: Option<&KalshiAuth>) -> Result<Self, Error> {
        Self::connect_to(PROD_WS, auth).await
    }

    /// Connect to the demo WebSocket API.
    pub async fn connect_demo(auth: Option<&KalshiAuth>) -> Result<Self, Error> {
        Self::connect_to(DEMO_WS, auth).await
    }

    /// Connect to a custom WebSocket URL.
    pub async fn connect_to(url: &str, auth: Option<&KalshiAuth>) -> Result<Self, Error> {
        let uri = url
            .parse::<tungstenite::http::Uri>()
            .map_err(|e| Error::Auth(format!("Invalid WebSocket URL: {e}")))?;

        let mut req_builder = tungstenite::http::Request::builder()
            .uri(url)
            .header("Host", uri.host().unwrap_or("api.elections.kalshi.com"))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            );

        if let Some(auth) = auth {
            let headers = auth.sign("GET", "/trade-api/ws/v2")?;
            req_builder = req_builder
                .header("KALSHI-ACCESS-KEY", &headers.key)
                .header("KALSHI-ACCESS-SIGNATURE", &headers.signature)
                .header("KALSHI-ACCESS-TIMESTAMP", &headers.timestamp);
        }

        let request = req_builder
            .body(())
            .map_err(|e| Error::Auth(format!("Failed to build WebSocket request: {e}")))?;

        let (ws, _response) = connect_async(request)
            .await
            .map_err(|e| Error::Auth(format!("WebSocket connection failed: {e}")))?;

        debug!("WebSocket connected to {url}");

        Ok(Self { ws, next_id: 1 })
    }

    /// Subscribe to one or more channels.
    ///
    /// For market-specific channels (orderbook_delta, trade), pass `market_tickers`.
    /// For global channels (ticker), pass `None`.
    pub async fn subscribe(
        &mut self,
        channels: &[Channel],
        market_tickers: Option<&[&str]>,
    ) -> Result<(), Error> {
        let cmd = WsCommand {
            id: self.next_id,
            cmd: "subscribe".to_string(),
            params: WsParams {
                channels: channels.iter().map(|c| c.as_str().to_string()).collect(),
                market_tickers: market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect()),
            },
        };
        self.next_id += 1;

        let msg = serde_json::to_string(&cmd).map_err(Error::Json)?;
        debug!(cmd = %msg, "sending subscribe");
        self.ws
            .send(tungstenite::Message::Text(msg.into()))
            .await
            .map_err(|e| Error::Auth(format!("Failed to send: {e}")))?;

        Ok(())
    }

    /// Unsubscribe from channels.
    pub async fn unsubscribe(
        &mut self,
        channels: &[Channel],
        market_tickers: Option<&[&str]>,
    ) -> Result<(), Error> {
        let cmd = WsCommand {
            id: self.next_id,
            cmd: "unsubscribe".to_string(),
            params: WsParams {
                channels: channels.iter().map(|c| c.as_str().to_string()).collect(),
                market_tickers: market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect()),
            },
        };
        self.next_id += 1;

        let msg = serde_json::to_string(&cmd).map_err(Error::Json)?;
        self.ws
            .send(tungstenite::Message::Text(msg.into()))
            .await
            .map_err(|e| Error::Auth(format!("Failed to send: {e}")))?;

        Ok(())
    }

    /// Receive the next event from the stream.
    ///
    /// Returns `None` if the connection is closed.
    pub async fn next(&mut self) -> Result<Option<Event>, Error> {
        loop {
            match self.ws.next().await {
                Some(Ok(tungstenite::Message::Text(text))) => match parse_event(&text) {
                    Ok(event) => return Ok(Some(event)),
                    Err(e) => {
                        warn!(error = %e, raw = %text, "failed to parse WebSocket message");
                        continue;
                    }
                },
                Some(Ok(tungstenite::Message::Ping(data))) => {
                    let _ = self.ws.send(tungstenite::Message::Pong(data)).await;
                    continue;
                }
                Some(Ok(tungstenite::Message::Close(_))) => return Ok(None),
                Some(Ok(_)) => continue, // Binary, Pong, Frame
                Some(Err(e)) => {
                    return Err(Error::Auth(format!("WebSocket error: {e}")));
                }
                None => return Ok(None),
            }
        }
    }

    /// Close the WebSocket connection.
    pub async fn close(mut self) -> Result<(), Error> {
        self.ws
            .close(None)
            .await
            .map_err(|e| Error::Auth(format!("Failed to close: {e}")))?;
        Ok(())
    }
}
