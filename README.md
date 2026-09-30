# Carry Engine — Binance Cash & Carry / Reverse Carry in Rust

Base project for collecting market history, ranking opportunities, checking Margin availability, and monitoring the risk of delta-neutral positions on Binance.

## Architecture

- `collector`: collects Spot/Futures prices, 24h volume, 24h price change, funding rate, next funding time, and basis.
- `analyzer`: uses historical data to generate separate Cash & Carry and Reverse Carry opportunities.
- `margin_scanner`: runs only for Reverse Carry candidates; checks `maxBorrowable` and Margin interest rates.
- `executor`: opens both legs of a position and includes basic rollback logic if the second leg fails.
- `risk_monitor`: independently monitors quantity drift and basis expansion outside the entry/exit logic.
- `MySQL`: stores market snapshots, opportunities, margin availability, positions, executions, and risk events.

## Safety

`LIVE_TRADING=false` is the default and blocks live order submission. Do not change it to `true` before validating quantity/notional filters for each symbol, the Futures account position mode, API permissions, and position-closing logic.

The code deliberately does not automatically close positions when the basis widens. Basis expansion may be temporary; the exit policy should be implemented separately using net PnL, accumulated funding, borrowing interest, fees, basis persistence, and hedge health.

## Database

```bash
mysql -u root -p < sql/schema.sql
```

## Configuration

```bash
cp .env.example .env
# Fill in DATABASE_URL and, for private endpoints, BINANCE_API_KEY/BINANCE_API_SECRET
```

## Run

```bash
cargo run
```

## Planned Flow

1. Collect data every 30 minutes to maintain a 15-day history.
2. Analyze all assets available on both Spot USDT and USDⓈ-M Perpetual Futures markets.
3. Maintain separate rankings for Cash & Carry and Reverse Carry opportunities.
4. Query Margin availability only for the Top N Reverse Carry candidates.
5. Reverse Carry: confirm borrow -> sell on Margin Spot -> buy Futures.
6. Cash & Carry: buy Spot -> sell Futures.
7. Run the risk monitor independently from the entry analysis.

## Before Production

Four components still need to be completed before using real funds:

- quantity rounding according to `LOT_SIZE`, `MARKET_LOT_SIZE`, `MIN_NOTIONAL/NOTIONAL`, and precision rules for each market;
- real fill reconciliation to guarantee a 1:1 hedge after partial executions;
- an explicit exit/position-closing module with failure recovery;
- accurate accounting for fee tiers, funding received/paid, and borrowing interest actually charged.

The project's priority should be market neutrality and capital preservation; profitability ranking comes after that.
