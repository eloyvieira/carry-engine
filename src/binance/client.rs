use crate::config::Config;
use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use reqwest::{Client, Method};
use serde_json::Value;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct BinanceClient {
    pub http: Client,
    pub cfg: Config,
}

impl BinanceClient {
    pub fn new(cfg: Config) -> Self {
        Self { http: Client::new(), cfg }
    }

    pub fn now_ms() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64
    }

    pub async fn public_get(&self, base: &str, path: &str, params: &[(&str, String)]) -> Result<Value> {
        let url = format!("{}{}", base, path);
        let res = self.http.get(url).query(params).send().await?;
        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() { return Err(anyhow!("Binance {}: {}", status, text)); }
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn signed_request(&self, method: Method, base: &str, path: &str, mut params: Vec<(String, String)>) -> Result<Value> {
        if self.cfg.api_key.is_empty() || self.cfg.api_secret.is_empty() {
            return Err(anyhow!("BINANCE_API_KEY / BINANCE_API_SECRET not configured"));
        }
        params.push(("timestamp".into(), Self::now_ms().to_string()));
        params.push(("recvWindow".into(), "5000".into()));

        // Binance signed endpoints require the percent-encoded payload to be signed.
        let encoded = serde_urlencoded::to_string(&params)?;
        let mut mac = HmacSha256::new_from_slice(self.cfg.api_secret.as_bytes())?;
        mac.update(encoded.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        let payload = format!("{}&signature={}", encoded, signature);
        let url = format!("{}{}", base, path);

        let req = match method {
            Method::GET => self.http.get(format!("{}?{}", url, payload)),
            Method::POST => self.http.post(url).header("Content-Type", "application/x-www-form-urlencoded").body(payload),
            Method::DELETE => self.http.delete(format!("{}?{}", url, payload)),
            _ => return Err(anyhow!("unsupported HTTP method")),
        }
        .header("X-MBX-APIKEY", &self.cfg.api_key);

        let res = req.send().await?;
        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() { return Err(anyhow!("Binance {}: {}", status, text)); }
        Ok(serde_json::from_str(&text)?)
    }
}
