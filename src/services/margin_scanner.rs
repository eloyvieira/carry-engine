use crate::{binance::client::BinanceClient, config::Config};
use anyhow::Result;
use serde_json::Value;
use sqlx::{MySql, Pool, Row};
use tracing::{info, warn};

pub async fn run_once(cfg: &Config, client: &BinanceClient, pool: &Pool<MySql>) -> Result<()> {
    let rows = sqlx::query(
        r#"SELECT o.symbol, s.base_asset, o.est_funding_apr
           FROM opportunities o
           JOIN (SELECT symbol, MAX(id) max_id FROM market_snapshots GROUP BY symbol) x ON x.symbol=o.symbol
           JOIN market_snapshots s ON s.id=x.max_id
           WHERE o.strategy='REVERSE_CARRY' AND o.eligible=1
           ORDER BY o.score DESC LIMIT ?"#,
    ).bind(cfg.top_n as i64).fetch_all(pool).await?;

    for row in rows {
        let symbol: String = row.try_get("symbol")?;
        let asset: String = row.try_get("base_asset")?;
        let est_funding_apr: f64 = row.try_get::<Option<f64>, _>("est_funding_apr")?.unwrap_or_default();

        let max_borrow = match client.max_borrowable(&asset).await {
            Ok(v) => v,
            Err(e) => { warn!(%symbol, %asset, error=%e, "max borrow query failed"); continue; }
        };

        let now = BinanceClient::now_ms();
        let start = now - 7 * 86_400_000;
        let history = client.margin_interest_history(&asset, start, now).await.unwrap_or_default();
        let avg_daily_rate = avg_interest(&history);
        let borrow_apr = avg_daily_rate * 365.0;
        let net_apr = est_funding_apr - borrow_apr;
        let available = max_borrow > 0.0 && borrow_apr <= cfg.max_margin_interest_apr && net_apr >= cfg.min_expected_net_apr;

        sqlx::query(
            r#"INSERT INTO margin_availability
            (symbol, asset, max_borrowable, avg_daily_interest_rate, est_borrow_apr, est_net_apr, available, checked_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, NOW())"#,
        ).bind(&symbol).bind(&asset).bind(max_borrow).bind(avg_daily_rate).bind(borrow_apr).bind(net_apr).bind(available)
        .execute(pool).await?;
        info!(%symbol, max_borrow, borrow_apr, net_apr, available, "margin checked");
    }
    Ok(())
}

fn avg_interest(v: &[Value]) -> f64 {
    let rates: Vec<f64> = v.iter().filter_map(|x| x.get("dailyInterestRate")?.as_str()?.parse().ok()).collect();
    if rates.is_empty() { 0.0 } else { rates.iter().sum::<f64>() / rates.len() as f64 }
}
