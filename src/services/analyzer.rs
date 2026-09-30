use crate::{config::Config, db, models::{Opportunity, Strategy}};
use anyhow::Result;
use sqlx::{MySql, Pool, Row};
use tracing::info;

pub async fn run_once(cfg: &Config, pool: &Pool<MySql>) -> Result<Vec<Opportunity>> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let cutoff_1d = now_ms - 86_400_000;
    let cutoff_3d = now_ms - 3 * 86_400_000;
    let cutoff_7d = now_ms - 7 * 86_400_000;
    let cutoff_hist = now_ms - cfg.history_days * 86_400_000;

    // Funding is calculated from actual historical funding events.
    // Basis/volume come from our own periodic snapshots.
    let rows = sqlx::query(
        r#"SELECT s.symbol,
            CAST((SELECT AVG(f1.funding_rate) FROM funding_rates f1 WHERE f1.symbol=s.symbol AND f1.funding_time_ms >= ?) AS DOUBLE) avg_f1,
            CAST((SELECT AVG(f3.funding_rate) FROM funding_rates f3 WHERE f3.symbol=s.symbol AND f3.funding_time_ms >= ?) AS DOUBLE) avg_f3,
            CAST((SELECT AVG(f7.funding_rate) FROM funding_rates f7 WHERE f7.symbol=s.symbol AND f7.funding_time_ms >= ?) AS DOUBLE) avg_f7,
            CAST((SELECT AVG(f7p.funding_rate > 0) FROM funding_rates f7p WHERE f7p.symbol=s.symbol AND f7p.funding_time_ms >= ?) AS DOUBLE) positive_ratio,
            CAST(AVG(CASE WHEN s.captured_at_ms >= ? THEN s.basis_pct END) AS DOUBLE) avg_basis1,
            CAST(STDDEV_POP(CASE WHEN s.captured_at_ms >= ? THEN s.basis_pct END) AS DOUBLE) std_basis7,
            CAST(MAX(s.volume_24h_quote) AS DOUBLE) volume24
        FROM market_snapshots s
        WHERE s.captured_at_ms >= ?
        GROUP BY s.symbol
        HAVING COUNT(*) >= 1"#,
    )
    .bind(cutoff_1d).bind(cutoff_3d).bind(cutoff_7d).bind(cutoff_7d)
    .bind(cutoff_1d).bind(cutoff_7d).bind(cutoff_hist)
    .fetch_all(pool).await?;

    let mut out = Vec::new();
    for r in rows {
        let symbol: String = r.try_get("symbol")?;
        let f1 = get_f64(&r, "avg_f1"); let f3 = get_f64(&r, "avg_f3"); let f7 = get_f64(&r, "avg_f7");
        let pos = get_f64(&r, "positive_ratio"); let b1 = get_f64(&r, "avg_basis1"); let bs = get_f64(&r, "std_basis7");
        let volume = get_f64(&r, "volume24");

        // Approximation only. Before live entry, calculate from the symbol's real funding schedule/history.
        let funding_apr_signed = f7 * 3.0 * 365.0;
        let fee_apr = 0.0;

        let cc_funding_apr = funding_apr_signed;
        let cc_net = cc_funding_apr - fee_apr;
        let cc_ok = cc_net >= cfg.min_expected_net_apr && pos >= 0.65 && bs <= cfg.max_basis_abs_pct && volume > 0.0;
        let cc = Opportunity {
            symbol: symbol.clone(), strategy: Strategy::CashAndCarry,
            score: cc_net * pos / (1.0 + bs.abs()), funding_rate_avg_1d: f1, funding_rate_avg_3d: f3,
            funding_rate_avg_7d: f7, funding_positive_ratio_7d: pos, basis_avg_1d_pct: b1, basis_std_7d_pct: bs,
            est_funding_apr: cc_funding_apr, est_borrow_apr: 0.0, est_fee_apr: fee_apr, est_net_apr: cc_net,
            break_even_days: None, eligible: cc_ok,
            reason: if cc_ok { "positive funding is persistent and basis volatility is acceptable".into() } else { "cash-and-carry filters not satisfied".into() },
        };
        db::save_opportunity(pool, &cc).await?;
        out.push(cc);

        let reverse_funding_apr = -funding_apr_signed;
        let reverse_ok = reverse_funding_apr >= cfg.min_expected_net_apr && pos <= 0.35 && bs <= cfg.max_basis_abs_pct && volume > 0.0;
        let rc = Opportunity {
            symbol: symbol.clone(), strategy: Strategy::ReverseCarry,
            score: reverse_funding_apr * (1.0 - pos) / (1.0 + bs.abs()),
            funding_rate_avg_1d: f1, funding_rate_avg_3d: f3, funding_rate_avg_7d: f7,
            funding_positive_ratio_7d: pos, basis_avg_1d_pct: b1, basis_std_7d_pct: bs,
            est_funding_apr: reverse_funding_apr, est_borrow_apr: 0.0, est_fee_apr: fee_apr,
            est_net_apr: reverse_funding_apr, break_even_days: None, eligible: reverse_ok,
            reason: if reverse_ok { "negative funding is persistent; margin cost still needs validation".into() } else { "reverse-carry historical filters not satisfied".into() },
        };
        db::save_opportunity(pool, &rc).await?;
        out.push(rc);
    }
    out.sort_by(|a,b| b.score.total_cmp(&a.score));
    info!(count=out.len(), "opportunities analyzed");
    Ok(out)
}

fn get_f64(row: &sqlx::mysql::MySqlRow, name: &str) -> f64 {
    row.try_get::<Option<f64>, _>(name).ok().flatten().unwrap_or_default()
}
