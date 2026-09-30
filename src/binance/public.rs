use super::client::BinanceClient;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SymbolMeta { pub symbol: String, pub base_asset: String, pub quote_asset: String }

impl BinanceClient {
    pub async fn common_perpetual_symbols(&self) -> Result<Vec<SymbolMeta>> {
        let spot = self.public_get(&self.cfg.spot_base_url, "/api/v3/exchangeInfo", &[]).await?;
        let fut = self.public_get(&self.cfg.futures_base_url, "/fapi/v1/exchangeInfo", &[]).await?;

        let mut future_set = std::collections::HashSet::new();
        if let Some(arr) = fut.get("symbols").and_then(Value::as_array) {
            for x in arr {
                if x.get("status").and_then(Value::as_str) == Some("TRADING")
                    && x.get("contractType").and_then(Value::as_str) == Some("PERPETUAL")
                    && x.get("quoteAsset").and_then(Value::as_str) == Some(self.cfg.quote_asset.as_str())
                {
                    if let Some(s) = x.get("symbol").and_then(Value::as_str) { future_set.insert(s.to_string()); }
                }
            }
        }

        let mut out = Vec::new();
        if let Some(arr) = spot.get("symbols").and_then(Value::as_array) {
            for x in arr {
                let symbol = x.get("symbol").and_then(Value::as_str).unwrap_or_default();
                let quote = x.get("quoteAsset").and_then(Value::as_str).unwrap_or_default();
                if x.get("status").and_then(Value::as_str) == Some("TRADING")
                    && quote == self.cfg.quote_asset
                    && future_set.contains(symbol)
                {
                    out.push(SymbolMeta {
                        symbol: symbol.to_string(),
                        base_asset: x.get("baseAsset").and_then(Value::as_str).unwrap_or_default().to_string(),
                        quote_asset: quote.to_string(),
                    });
                }
            }
        }
        Ok(out)
    }

    pub async fn spot_24h_map(&self) -> Result<HashMap<String, Value>> {
        let v = self.public_get(&self.cfg.spot_base_url, "/api/v3/ticker/24hr", &[]).await?;
        array_map(v)
    }

    pub async fn spot_book_map(&self) -> Result<HashMap<String, Value>> {
        let v = self.public_get(&self.cfg.spot_base_url, "/api/v3/ticker/bookTicker", &[]).await?;
        array_map(v)
    }

    pub async fn futures_book_map(&self) -> Result<HashMap<String, Value>> {
        let v = self.public_get(&self.cfg.futures_base_url, "/fapi/v1/ticker/bookTicker", &[]).await?;
        array_map(v)
    }

    pub async fn premium_index_map(&self) -> Result<HashMap<String, Value>> {
        let v = self.public_get(&self.cfg.futures_base_url, "/fapi/v1/premiumIndex", &[]).await?;
        array_map(v)
    }

    pub async fn funding_history(&self, symbol: &str, start_time_ms: i64, limit: usize) -> Result<Vec<Value>> {
        let params = vec![
            ("symbol", symbol.to_string()),
            ("startTime", start_time_ms.to_string()),
            ("limit", limit.min(1000).to_string()),
        ];
        let v = self.public_get(&self.cfg.futures_base_url, "/fapi/v1/fundingRate", &params).await?;
        Ok(v.as_array().cloned().ok_or_else(|| anyhow!("unexpected funding history response"))?)
    }
}

fn array_map(v: Value) -> Result<HashMap<String, Value>> {
    let mut map = HashMap::new();
    let arr = v.as_array().ok_or_else(|| anyhow!("expected array"))?;
    for item in arr {
        if let Some(symbol) = item.get("symbol").and_then(Value::as_str) {
            map.insert(symbol.to_string(), item.clone());
        }
    }
    Ok(map)
}
