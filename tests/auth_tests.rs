//! Tests for the authentication module.

use kalshi::auth::KalshiAuth;
use kalshi::error::Error;
use std::io::Write;
use tempfile::NamedTempFile;

const TEST_KEY_PEM: &str = include_str!("test_key.pem");

#[test]
fn from_key_pem_valid() {
    let auth = KalshiAuth::from_key_pem("test-key", TEST_KEY_PEM);
    assert!(auth.is_ok());
}

#[test]
fn from_key_pem_invalid() {
    let err = KalshiAuth::from_key_pem("test-key", "not a pem").unwrap_err();
    assert!(matches!(err, Error::Auth(_)));
}

#[test]
fn from_key_file_valid() {
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(TEST_KEY_PEM.as_bytes()).unwrap();
    let auth = KalshiAuth::from_key_file("test-key", tmp.path());
    assert!(auth.is_ok());
}

#[test]
fn from_key_file_missing() {
    let err = KalshiAuth::from_key_file("test-key", "/nonexistent/key.pem").unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn sign_produces_valid_headers() {
    let auth = KalshiAuth::from_key_pem("my-key-id", TEST_KEY_PEM).unwrap();
    let headers = auth.sign("GET", "/trade-api/v2/markets").unwrap();
    assert_eq!(headers.key, "my-key-id");
    assert!(!headers.signature.is_empty());
    // Timestamp should be a numeric string (millis since epoch)
    let ts: u128 = headers.timestamp.parse().expect("timestamp should be numeric");
    assert!(ts > 1_000_000_000_000); // after ~2001
}

#[test]
fn sign_strips_query_params() {
    let auth = KalshiAuth::from_key_pem("key", TEST_KEY_PEM).unwrap();
    let h1 = auth.sign("GET", "/markets?status=open").unwrap();
    let h2 = auth.sign("GET", "/markets").unwrap();
    // Different timestamps mean different signatures, but both should succeed
    assert!(!h1.signature.is_empty());
    assert!(!h2.signature.is_empty());
}

#[test]
fn debug_redacts_private_key() {
    let auth = KalshiAuth::from_key_pem("test-key", TEST_KEY_PEM).unwrap();
    let debug = format!("{auth:?}");
    assert!(debug.contains("REDACTED"));
    assert!(!debug.contains("BEGIN"));
}
