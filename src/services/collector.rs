use crate::{binance::client::BinanceClient, db, models::MarketSnapshot};
use anyhow::Result;
use serde_json::Value;
use sqlx::{MySql, Pool};
use tracing::{info, warn};

pub async fn run_once(client: &BinanceClient, pool: &Pool<MySql>) -> Result<usize> {
    let symbols = client.common_perpetual_symbols().await?;
    let (spot24, spotbook, futbook, premium) = tokio::try_join!(
        client.spot_24h_map(), client.spot_book_map(), client.futures_book_map(), client.premium_index_map()
    )?;
    let mut saved = 0;
    let now = BinanceClient::now_ms();

    for meta in symbols {
        let Some(s24) = spot24.get(&meta.symbol) else { continue };
        let Some(sb) = spotbook.get(&meta.symbol) else { continue };
        let Some(fb) = futbook.get(&meta.symbol) else { continue };
        let Some(pm) = premium.get(&meta.symbol) else { continue };

        let spot_bid = num(sb, "bidPrice");
        let spot_ask = num(sb, "askPrice");
        let spot_last = num(s24, "lastPrice");
        let futures_bid = num(fb, "bidPrice");
        let futures_ask = num(fb, "askPrice");
        let futures_mark = num(pm, "markPrice");
        let index_price = num(pm, "indexPrice");
        if spot_last <= 0.0 || futures_mark <= 0.0 { continue; }
        let basis_pct = (futures_mark / spot_last - 1.0) * 100.0;
        let snap = MarketSnapshot {
            symbol: meta.symbol.clone(), base_asset: meta.base_asset, quote_asset: meta.quote_asset,
            spot_bid, spot_ask, spot_last, futures_bid, futures_ask, futures_mark, index_price,
            volume_24h_quote: num(s24, "quoteVolume"), change_24h_pct: num(s24, "priceChangePercent"),
            funding_rate: num(pm, "lastFundingRate"),
            next_funding_time_ms: pm.get("nextFundingTime").and_then(Value::as_i64).unwrap_or(0),
            basis_pct, captured_at_ms: now,
        };
        if let Err(e) = db::save_snapshot(pool, &snap).await { warn!(symbol=%snap.symbol, error=%e, "snapshot save failed"); }
        else { saved += 1; }
    }
    info!(saved, "market snapshots saved");
    Ok(saved)
}

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_str).and_then(|s| s.parse().ok()).unwrap_or_default()
}
