use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub spot_base_url: String,
    pub futures_base_url: String,
    pub quote_asset: String,
    pub collect_interval_seconds: u64,
    pub analysis_interval_seconds: u64,
    pub margin_scan_interval_seconds: u64,
    pub risk_interval_seconds: u64,
    pub top_n: usize,
    pub history_days: i64,
    pub live_trading: bool,
    pub max_position_usdt: f64,
    pub max_total_exposure_usdt: f64,
    pub min_expected_net_apr: f64,
    pub max_basis_abs_pct: f64,
    pub max_hedge_drift_pct: f64,
    pub max_margin_interest_apr: f64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            api_key: env::var("BINANCE_API_KEY").unwrap_or_default(),
            api_secret: env::var("BINANCE_API_SECRET").unwrap_or_default(),
            spot_base_url: env::var("BINANCE_SPOT_BASE_URL").unwrap_or_else(|_| "https://api.binance.com".into()),
            futures_base_url: env::var("BINANCE_FUTURES_BASE_URL").unwrap_or_else(|_| "https://fapi.binance.com".into()),
            quote_asset: env::var("QUOTE_ASSET").unwrap_or_else(|_| "USDT".into()),
            collect_interval_seconds: env_u64("COLLECT_INTERVAL_SECONDS", 1800),
            analysis_interval_seconds: env_u64("ANALYSIS_INTERVAL_SECONDS", 300),
            margin_scan_interval_seconds: env_u64("MARGIN_SCAN_INTERVAL_SECONDS", 60),
            risk_interval_seconds: env_u64("RISK_INTERVAL_SECONDS", 10),
            top_n: env_usize("TOP_N", 10),
            history_days: env_i64("HISTORY_DAYS", 15),
            live_trading: env_bool("LIVE_TRADING", false),
            max_position_usdt: env_f64("MAX_POSITION_USDT", 100.0),
            max_total_exposure_usdt: env_f64("MAX_TOTAL_EXPOSURE_USDT", 1000.0),
            min_expected_net_apr: env_f64("MIN_EXPECTED_NET_APR", 0.05),
            max_basis_abs_pct: env_f64("MAX_BASIS_ABS_PCT", 2.0),
            max_hedge_drift_pct: env_f64("MAX_HEDGE_DRIFT_PCT", 0.50),
            max_margin_interest_apr: env_f64("MAX_MARGIN_INTEREST_APR", 0.25),
        })
    }
}

fn env_u64(k: &str, d: u64) -> u64 { env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d) }
fn env_usize(k: &str, d: usize) -> usize { env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d) }
fn env_i64(k: &str, d: i64) -> i64 { env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d) }
fn env_f64(k: &str, d: f64) -> f64 { env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d) }
fn env_bool(k: &str, d: bool) -> bool { env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d) }
