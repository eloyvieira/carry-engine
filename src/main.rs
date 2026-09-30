mod binance;
mod config;
mod db;
mod models;
mod services;

use anyhow::Result;
use binance::client::BinanceClient;
use config::Config;
use std::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let cfg = Config::from_env()?;
    let pool = db::connect(&cfg.database_url).await?;
    let client = BinanceClient::new(cfg.clone());

    info!(live_trading=cfg.live_trading, "carry engine started");

    // Backfill the last N days of actual funding events before starting the periodic loops.
    if let Err(e) = services::funding_backfill::run_once(&client, &pool, cfg.history_days).await {
        error!(error=%e, "initial funding backfill failed");
    }

    let collector_cfg = cfg.clone(); let collector_client = client.clone(); let collector_pool = pool.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(collector_cfg.collect_interval_seconds));
        loop { tick.tick().await; if let Err(e) = services::collector::run_once(&collector_client, &collector_pool).await { error!(error=%e, "collector failed"); } }
    });

    let analyzer_cfg = cfg.clone(); let analyzer_pool = pool.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(analyzer_cfg.analysis_interval_seconds));
        loop { tick.tick().await; if let Err(e) = services::analyzer::run_once(&analyzer_cfg, &analyzer_pool).await { error!(error=%e, "analyzer failed"); } }
    });

    let margin_cfg = cfg.clone(); let margin_client = client.clone(); let margin_pool = pool.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(margin_cfg.margin_scan_interval_seconds));
        loop { tick.tick().await; if let Err(e) = services::margin_scanner::run_once(&margin_cfg, &margin_client, &margin_pool).await { error!(error=%e, "margin scanner failed"); } }
    });

    let risk_cfg = cfg.clone(); let risk_pool = pool.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(risk_cfg.risk_interval_seconds));
        loop { tick.tick().await; if let Err(e) = services::risk_monitor::run_once(&risk_cfg, &risk_pool).await { error!(error=%e, "risk monitor failed"); } }
    });

    tokio::signal::ctrl_c().await?;
    info!("shutdown signal received");
    Ok(())
}
