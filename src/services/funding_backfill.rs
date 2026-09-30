use crate::binance::client::BinanceClient;
use anyhow::Result;
use serde_json::Value;
use sqlx::{MySql, Pool};
use tracing::{info, warn};

pub async fn run_once(client: &BinanceClient, pool: &Pool<MySql>, history_days: i64) -> Result<()> {
    let symbols = client.common_perpetual_symbols().await?;
    let start_ms = BinanceClient::now_ms() - history_days * 86_400_000;
    let mut inserted = 0usize;

    for meta in symbols {
        match client.funding_history(&meta.symbol, start_ms, 1000).await {
            Ok(items) => {
                for item in items {
                    let funding_time = item.get("fundingTime").and_then(Value::as_i64).unwrap_or(0);
                    let funding_rate = item.get("fundingRate").and_then(Value::as_str).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                    let mark_price = item.get("markPrice").and_then(Value::as_str).and_then(|v| v.parse::<f64>().ok());
                    if funding_time == 0 { continue; }
                    sqlx::query(
                        "INSERT INTO funding_rates (symbol, funding_time_ms, funding_rate, mark_price) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE funding_rate=VALUES(funding_rate), mark_price=VALUES(mark_price)"
                    ).bind(&meta.symbol).bind(funding_time).bind(funding_rate).bind(mark_price).execute(pool).await?;
                    inserted += 1;
                }
            }
            Err(e) => warn!(symbol=%meta.symbol, error=%e, "funding backfill failed"),
        }
        tokio::time::sleep(std::time::Duration::from_millis(75)).await;
    }
    info!(inserted, "funding history backfill complete");
    Ok(())
}
