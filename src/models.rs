use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub spot_bid: f64,
    pub spot_ask: f64,
    pub spot_last: f64,
    pub futures_bid: f64,
    pub futures_ask: f64,
    pub futures_mark: f64,
    pub index_price: f64,
    pub volume_24h_quote: f64,
    pub change_24h_pct: f64,
    pub funding_rate: f64,
    pub next_funding_time_ms: i64,
    pub basis_pct: f64,
    pub captured_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub symbol: String,
    pub strategy: Strategy,
    pub score: f64,
    pub funding_rate_avg_1d: f64,
    pub funding_rate_avg_3d: f64,
    pub funding_rate_avg_7d: f64,
    pub funding_positive_ratio_7d: f64,
    pub basis_avg_1d_pct: f64,
    pub basis_std_7d_pct: f64,
    pub est_funding_apr: f64,
    pub est_borrow_apr: f64,
    pub est_fee_apr: f64,
    pub est_net_apr: f64,
    pub break_even_days: Option<f64>,
    pub eligible: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Strategy {
    CashAndCarry,
    ReverseCarry,
}

impl Strategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CashAndCarry => "CASH_AND_CARRY",
            Self::ReverseCarry => "REVERSE_CARRY",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PositionHealth {
    pub position_id: i64,
    pub symbol: String,
    pub strategy: String,
    pub spot_qty: f64,
    pub futures_qty: f64,
    pub spot_price: f64,
    pub futures_price: f64,
    pub hedge_drift_pct: f64,
    pub basis_pct: f64,
    pub total_pnl_usdt: f64,
}
