# Carry Engine — Binance Cash & Carry / Reverse Carry in Rust

Base project for collecting market history, ranking opportunities, checking Margin availability, and monitoring the risk of delta-neutral positions on Binance.

## Architecture

- `collector`: collects Spot/Futures prices, 24h volume, 24h price change, funding rate, next funding time, and basis.
- `analyzer`: uses historical data to generate separate Cash & Carry and Reverse Carry opportunities.
- `margin_scanner`: runs only for Reverse Carry candidates; checks `maxBorrowable` and Margin interest rates.
- `executor`: opens both legs of a position and includes basic rollback logic if the second leg fails.
- `risk_monitor`: independently monitors quantity drift and basis expansion outside the entry/exit logic.
- `MySQL`: stores market snapshots, opportunities, margin availability, positions, executions, and risk events.

## Database

```bash
mysql -u root -p < sql/schema.sql
```

## Configuration

Safety:

```text
`LIVE_TRADING=false` is enabled by default and prevents live orders.
```

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Create the environment file:

```bash
cp .env.example .env
```

Load environment variables and start the application:

```bash
set -a
source .env
set +a
```

Run:

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

Before using real funds, complete:

- quantity rounding according to `LOT_SIZE`, `MARKET_LOT_SIZE`, `MIN_NOTIONAL/NOTIONAL`, and precision rules for each market;
- real fill reconciliation to guarantee a 1:1 hedge after partial executions;
- position closing and failure recovery;
- accurate fees, funding, and borrowing cost accounting.

The priority is market neutrality and capital preservation.
