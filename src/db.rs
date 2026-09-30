use crate::models::{MarketSnapshot, Opportunity};
use anyhow::Result;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

pub async fn connect(url: &str) -> Result<Pool<MySql>> {
    Ok(MySqlPoolOptions::new().max_connections(10).connect(url).await?)
}

pub async fn save_snapshot(pool: &Pool<MySql>, s: &MarketSnapshot) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO market_snapshots
        (symbol, base_asset, quote_asset, spot_bid, spot_ask, spot_last, futures_bid, futures_ask,
         futures_mark, index_price, volume_24h_quote, change_24h_pct, funding_rate,
         next_funding_time_ms, basis_pct, captured_at_ms)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
         spot_bid=VALUES(spot_bid), spot_ask=VALUES(spot_ask), spot_last=VALUES(spot_last),
         futures_bid=VALUES(futures_bid), futures_ask=VALUES(futures_ask), futures_mark=VALUES(futures_mark),
         index_price=VALUES(index_price), volume_24h_quote=VALUES(volume_24h_quote),
         change_24h_pct=VALUES(change_24h_pct), funding_rate=VALUES(funding_rate),
         next_funding_time_ms=VALUES(next_funding_time_ms), basis_pct=VALUES(basis_pct)"#,
    )
    .bind(&s.symbol).bind(&s.base_asset).bind(&s.quote_asset)
    .bind(s.spot_bid).bind(s.spot_ask).bind(s.spot_last)
    .bind(s.futures_bid).bind(s.futures_ask).bind(s.futures_mark)
    .bind(s.index_price).bind(s.volume_24h_quote).bind(s.change_24h_pct)
    .bind(s.funding_rate).bind(s.next_funding_time_ms).bind(s.basis_pct).bind(s.captured_at_ms)
    .execute(pool).await?;
    Ok(())
}

pub async fn save_opportunity(pool: &Pool<MySql>, o: &Opportunity) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO opportunities
        (symbol, strategy, score, funding_rate_avg_1d, funding_rate_avg_3d, funding_rate_avg_7d,
         funding_positive_ratio_7d, basis_avg_1d_pct, basis_std_7d_pct, est_funding_apr,
         est_borrow_apr, est_fee_apr, est_net_apr, break_even_days, eligible, reason, analyzed_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW())
        ON DUPLICATE KEY UPDATE
         score=VALUES(score), funding_rate_avg_1d=VALUES(funding_rate_avg_1d),
         funding_rate_avg_3d=VALUES(funding_rate_avg_3d), funding_rate_avg_7d=VALUES(funding_rate_avg_7d),
         funding_positive_ratio_7d=VALUES(funding_positive_ratio_7d), basis_avg_1d_pct=VALUES(basis_avg_1d_pct),
         basis_std_7d_pct=VALUES(basis_std_7d_pct), est_funding_apr=VALUES(est_funding_apr),
         est_borrow_apr=VALUES(est_borrow_apr), est_fee_apr=VALUES(est_fee_apr), est_net_apr=VALUES(est_net_apr),
         break_even_days=VALUES(break_even_days), eligible=VALUES(eligible), reason=VALUES(reason), analyzed_at=NOW()"#,
    )
    .bind(&o.symbol).bind(o.strategy.as_str()).bind(o.score)
    .bind(o.funding_rate_avg_1d).bind(o.funding_rate_avg_3d).bind(o.funding_rate_avg_7d)
    .bind(o.funding_positive_ratio_7d).bind(o.basis_avg_1d_pct).bind(o.basis_std_7d_pct)
    .bind(o.est_funding_apr).bind(o.est_borrow_apr).bind(o.est_fee_apr).bind(o.est_net_apr)
    .bind(o.break_even_days).bind(o.eligible).bind(&o.reason)
    .execute(pool).await?;
    Ok(())
}
