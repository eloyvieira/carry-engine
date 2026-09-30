use crate::{binance::client::BinanceClient, config::Config, models::Strategy};
use anyhow::{anyhow, Result};
use sqlx::{MySql, Pool};
use tracing::info;

pub struct Executor<'a> {
    cfg: &'a Config,
    client: &'a BinanceClient,
    pool: &'a Pool<MySql>,
}

impl<'a> Executor<'a> {
    pub fn new(cfg: &'a Config, client: &'a BinanceClient, pool: &'a Pool<MySql>) -> Self { Self { cfg, client, pool } }

    pub async fn open_reverse_carry(&self, symbol: &str, asset: &str, qty: f64) -> Result<()> {
        self.require_live()?;
        let max_borrow = self.client.max_borrowable(asset).await?;
        if max_borrow < qty { return Err(anyhow!("insufficient margin inventory: max_borrowable={max_borrow}, qty={qty}")); }

        // Safety ordering: borrow first; never open the futures leg before borrow is confirmed.
        let borrow = self.client.borrow_margin(asset, qty).await?;
        self.log_exec(symbol, "REVERSE_CARRY", "MARGIN_BORROW", "BORROW", qty, &borrow).await?;

        let spot = match self.client.margin_market_order(symbol, "SELL", qty).await {
            Ok(v) => v,
            Err(e) => {
                let _ = self.client.repay_margin(asset, qty).await;
                return Err(e);
            }
        };
        self.log_exec(symbol, "REVERSE_CARRY", "MARGIN_SPOT", "SELL", qty, &spot).await?;

        match self.client.futures_market_order(symbol, "BUY", qty, false).await {
            Ok(fut) => {
                self.log_exec(symbol, "REVERSE_CARRY", "FUTURES", "BUY", qty, &fut).await?;
                self.create_position(symbol, Strategy::ReverseCarry, qty).await?;
                Ok(())
            }
            Err(e) => {
                // Compensating action: buy back the short and repay the loan.
                let _ = self.client.margin_market_order(symbol, "BUY", qty).await;
                let _ = self.client.repay_margin(asset, qty).await;
                Err(anyhow!("futures leg failed; attempted rollback: {e}"))
            }
        }
    }

    pub async fn open_cash_and_carry(&self, symbol: &str, qty: f64) -> Result<()> {
        self.require_live()?;
        let spot = self.client.spot_market_order(symbol, "BUY", qty).await?;
        self.log_exec(symbol, "CASH_AND_CARRY", "SPOT", "BUY", qty, &spot).await?;
        match self.client.futures_market_order(symbol, "SELL", qty, false).await {
            Ok(fut) => {
                self.log_exec(symbol, "CASH_AND_CARRY", "FUTURES", "SELL", qty, &fut).await?;
                self.create_position(symbol, Strategy::CashAndCarry, qty).await?;
                Ok(())
            }
            Err(e) => {
                let _ = self.client.spot_market_order(symbol, "SELL", qty).await;
                Err(anyhow!("futures leg failed; attempted spot rollback: {e}"))
            }
        }
    }

    fn require_live(&self) -> Result<()> {
        if !self.cfg.live_trading { return Err(anyhow!("LIVE_TRADING=false; execution blocked")); }
        Ok(())
    }

    async fn log_exec(&self, symbol: &str, strategy: &str, venue: &str, side: &str, qty: f64, raw: &serde_json::Value) -> Result<()> {
        sqlx::query("INSERT INTO executions (symbol, strategy, venue, side, quantity, raw_response, created_at) VALUES (?, ?, ?, ?, ?, ?, NOW())")
            .bind(symbol).bind(strategy).bind(venue).bind(side).bind(qty).bind(raw.to_string()).execute(self.pool).await?;
        Ok(())
    }

    async fn create_position(&self, symbol: &str, strategy: Strategy, qty: f64) -> Result<()> {
        sqlx::query("INSERT INTO positions (symbol, strategy, status, spot_qty, futures_qty, opened_at) VALUES (?, ?, 'OPEN', ?, ?, NOW())")
            .bind(symbol).bind(strategy.as_str()).bind(qty).bind(qty).execute(self.pool).await?;
        info!(%symbol, strategy=%strategy.as_str(), qty, "position opened");
        Ok(())
    }
}
