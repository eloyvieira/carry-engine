use crate::config::Config;
use anyhow::Result;
use sqlx::{MySql, Pool, Row};
use tracing::warn;

pub async fn run_once(cfg: &Config, pool: &Pool<MySql>) -> Result<()> {
    let rows = sqlx::query(
        r#"SELECT p.id, p.symbol, p.strategy, p.spot_qty, p.futures_qty,
                  s.spot_last, s.futures_mark, s.basis_pct
           FROM positions p
           JOIN (SELECT symbol, MAX(id) max_id FROM market_snapshots GROUP BY symbol) x ON x.symbol=p.symbol
           JOIN market_snapshots s ON s.id=x.max_id
           WHERE p.status='OPEN'"#,
    ).fetch_all(pool).await?;

    for r in rows {
        let id: i64 = r.try_get("id")?;
        let symbol: String = r.try_get("symbol")?;
        let spot_qty: f64 = r.try_get::<f64,_>("spot_qty")?;
        let futures_qty: f64 = r.try_get::<f64,_>("futures_qty")?;
        let basis_pct: f64 = r.try_get::<f64,_>("basis_pct")?;
        let denom = spot_qty.abs().max(1e-12);
        let hedge_drift_pct = ((futures_qty.abs() - spot_qty.abs()).abs() / denom) * 100.0;

        let mut severity = None;
        let mut reason = Vec::new();
        if hedge_drift_pct > cfg.max_hedge_drift_pct { severity = Some("CRITICAL"); reason.push(format!("hedge drift {:.4}%", hedge_drift_pct)); }
        if basis_pct.abs() > cfg.max_basis_abs_pct { severity.get_or_insert("WARN"); reason.push(format!("basis {:.4}%", basis_pct)); }

        if let Some(level) = severity {
            let reason = reason.join("; ");
            sqlx::query("INSERT INTO risk_events (position_id, symbol, severity, event_type, details, created_at) VALUES (?, ?, ?, 'POSITION_HEALTH', ?, NOW())")
                .bind(id).bind(&symbol).bind(level).bind(&reason).execute(pool).await?;
            warn!(position_id=id, %symbol, level, %reason, "risk event");
        }
    }
    Ok(())
}
