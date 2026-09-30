use super::client::BinanceClient;
use anyhow::{anyhow, Result};
use reqwest::Method;
use serde_json::Value;

impl BinanceClient {
    pub async fn max_borrowable(&self, asset: &str) -> Result<f64> {
        let v = self.signed_request(
            Method::GET,
            &self.cfg.spot_base_url,
            "/sapi/v1/margin/maxBorrowable",
            vec![("asset".into(), asset.into())],
        ).await?;
        parse_num(v.get("amount"))
    }

    pub async fn margin_interest_history(&self, asset: &str, start_ms: i64, end_ms: i64) -> Result<Vec<Value>> {
        let v = self.signed_request(
            Method::GET,
            &self.cfg.spot_base_url,
            "/sapi/v1/margin/interestRateHistory",
            vec![
                ("asset".into(), asset.into()),
                ("startTime".into(), start_ms.to_string()),
                ("endTime".into(), end_ms.to_string()),
            ],
        ).await?;
        Ok(v.as_array().cloned().unwrap_or_default())
    }

    pub async fn borrow_margin(&self, asset: &str, amount: f64) -> Result<Value> {
        self.signed_request(
            Method::POST,
            &self.cfg.spot_base_url,
            "/sapi/v1/margin/borrow-repay",
            vec![
                ("asset".into(), asset.into()),
                ("isIsolated".into(), "FALSE".into()),
                ("amount".into(), trim_float(amount)),
                ("type".into(), "BORROW".into()),
            ],
        ).await
    }

    pub async fn repay_margin(&self, asset: &str, amount: f64) -> Result<Value> {
        self.signed_request(
            Method::POST,
            &self.cfg.spot_base_url,
            "/sapi/v1/margin/borrow-repay",
            vec![
                ("asset".into(), asset.into()),
                ("isIsolated".into(), "FALSE".into()),
                ("amount".into(), trim_float(amount)),
                ("type".into(), "REPAY".into()),
            ],
        ).await
    }

    pub async fn margin_market_order(&self, symbol: &str, side: &str, quantity: f64) -> Result<Value> {
        self.signed_request(
            Method::POST,
            &self.cfg.spot_base_url,
            "/sapi/v1/margin/order",
            vec![
                ("symbol".into(), symbol.into()),
                ("isIsolated".into(), "FALSE".into()),
                ("side".into(), side.into()),
                ("type".into(), "MARKET".into()),
                ("quantity".into(), trim_float(quantity)),
                ("newOrderRespType".into(), "FULL".into()),
                ("sideEffectType".into(), "NO_SIDE_EFFECT".into()),
            ],
        ).await
    }

    pub async fn spot_market_order(&self, symbol: &str, side: &str, quantity: f64) -> Result<Value> {
        self.signed_request(
            Method::POST,
            &self.cfg.spot_base_url,
            "/api/v3/order",
            vec![
                ("symbol".into(), symbol.into()),
                ("side".into(), side.into()),
                ("type".into(), "MARKET".into()),
                ("quantity".into(), trim_float(quantity)),
                ("newOrderRespType".into(), "FULL".into()),
            ],
        ).await
    }

    pub async fn futures_market_order(&self, symbol: &str, side: &str, quantity: f64, reduce_only: bool) -> Result<Value> {
        self.signed_request(
            Method::POST,
            &self.cfg.futures_base_url,
            "/fapi/v1/order",
            vec![
                ("symbol".into(), symbol.into()),
                ("side".into(), side.into()),
                ("type".into(), "MARKET".into()),
                ("quantity".into(), trim_float(quantity)),
                ("reduceOnly".into(), reduce_only.to_string()),
                ("newOrderRespType".into(), "RESULT".into()),
            ],
        ).await
    }
}

fn parse_num(v: Option<&Value>) -> Result<f64> {
    v.and_then(Value::as_str).ok_or_else(|| anyhow!("missing numeric field"))?.parse().map_err(Into::into)
}

fn trim_float(v: f64) -> String {
    let s = format!("{:.12}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
