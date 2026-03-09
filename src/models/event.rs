use serde::{Deserialize, Serialize};

/// A Kalshi event (contains one or more markets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_ticker: String,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub mutually_exclusive: Option<bool>,
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub markets: Vec<String>,
}

/// Paginated events response.
#[derive(Debug, Clone, Deserialize)]
pub struct EventsResponse {
    pub events: Vec<Event>,
    pub cursor: Option<String>,
}

/// Single event response.
#[derive(Debug, Clone, Deserialize)]
pub struct EventResponse {
    pub event: Event,
}
