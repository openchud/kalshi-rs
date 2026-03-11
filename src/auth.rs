use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rsa::RsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::{BlindedSigningKey, Signature};
use rsa::sha2::Sha256;
use rsa::signature::RandomizedSigner;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Error;

/// Authentication handler for Kalshi API requests.
///
/// Uses RSA-PSS signatures as required by the Kalshi API.
#[derive(Clone)]
pub struct KalshiAuth {
    api_key_id: String,
    private_key: RsaPrivateKey,
}

impl KalshiAuth {
    /// Create auth from an API key ID and a PEM-encoded private key file.
    pub fn from_key_file(
        api_key_id: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let pem = std::fs::read_to_string(path)?;
        Self::from_key_pem(api_key_id, &pem)
    }

    /// Create auth from an API key ID and a PEM-encoded private key string.
    pub fn from_key_pem(api_key_id: impl Into<String>, pem: &str) -> Result<Self, Error> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(pem)
            .map_err(|e| Error::Auth(format!("Failed to parse RSA private key: {e}")))?;
        Ok(Self {
            api_key_id: api_key_id.into(),
            private_key,
        })
    }

    /// Generate authentication headers for a request.
    ///
    /// Returns (key, signature, timestamp) header values.
    /// Generate authentication headers for a request.
    ///
    /// Returns [`AuthHeaders`] containing the key, signature, and timestamp.
    pub fn sign(&self, method: &str, path: &str) -> Result<AuthHeaders, Error> {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Auth(format!("Clock error: {e}")))?
            .as_millis()
            .to_string();

        // Strip query params for signing
        let sign_path = path.split('?').next().unwrap_or(path);
        let message = format!("{timestamp_ms}{method}{sign_path}");

        let signing_key = BlindedSigningKey::<Sha256>::new(self.private_key.clone());
        let mut rng = rand::thread_rng();
        let signature: Signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
        let sig_bytes: Box<[u8]> = signature.into();

        Ok(AuthHeaders {
            key: self.api_key_id.clone(),
            signature: BASE64.encode(&*sig_bytes),
            timestamp: timestamp_ms,
        })
    }
}

/// Auth header values to attach to requests.
pub struct AuthHeaders {
    pub key: String,
    pub signature: String,
    pub timestamp: String,
}

impl std::fmt::Debug for KalshiAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KalshiAuth")
            .field("api_key_id", &self.api_key_id)
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}
