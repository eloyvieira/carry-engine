CREATE DATABASE IF NOT EXISTS carry_engine CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE carry_engine;

CREATE TABLE IF NOT EXISTS market_snapshots (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    symbol VARCHAR(30) NOT NULL,
    base_asset VARCHAR(20) NOT NULL,
    quote_asset VARCHAR(20) NOT NULL,
    spot_bid DECIMAL(30,12) NOT NULL DEFAULT 0,
    spot_ask DECIMAL(30,12) NOT NULL DEFAULT 0,
    spot_last DECIMAL(30,12) NOT NULL DEFAULT 0,
    futures_bid DECIMAL(30,12) NOT NULL DEFAULT 0,
    futures_ask DECIMAL(30,12) NOT NULL DEFAULT 0,
    futures_mark DECIMAL(30,12) NOT NULL DEFAULT 0,
    index_price DECIMAL(30,12) NOT NULL DEFAULT 0,
    volume_24h_quote DECIMAL(36,8) NOT NULL DEFAULT 0,
    change_24h_pct DECIMAL(18,8) NOT NULL DEFAULT 0,
    funding_rate DECIMAL(20,12) NOT NULL DEFAULT 0,
    next_funding_time_ms BIGINT NOT NULL DEFAULT 0,
    basis_pct DECIMAL(18,8) NOT NULL DEFAULT 0,
    captured_at_ms BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uq_snapshot (symbol, captured_at_ms),
    KEY ix_snapshot_symbol_time (symbol, captured_at_ms),
    KEY ix_snapshot_time (captured_at_ms)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS funding_rates (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    symbol VARCHAR(30) NOT NULL,
    funding_time_ms BIGINT NOT NULL,
    funding_rate DECIMAL(20,12) NOT NULL,
    mark_price DECIMAL(30,12) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uq_funding (symbol, funding_time_ms),
    KEY ix_funding_symbol_time (symbol, funding_time_ms)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS opportunities (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    symbol VARCHAR(30) NOT NULL,
    strategy ENUM('CASH_AND_CARRY','REVERSE_CARRY') NOT NULL,
    score DECIMAL(20,10) NOT NULL DEFAULT 0,
    funding_rate_avg_1d DECIMAL(20,12) NOT NULL DEFAULT 0,
    funding_rate_avg_3d DECIMAL(20,12) NOT NULL DEFAULT 0,
    funding_rate_avg_7d DECIMAL(20,12) NOT NULL DEFAULT 0,
    funding_positive_ratio_7d DECIMAL(12,8) NOT NULL DEFAULT 0,
    basis_avg_1d_pct DECIMAL(18,8) NOT NULL DEFAULT 0,
    basis_std_7d_pct DECIMAL(18,8) NOT NULL DEFAULT 0,
    est_funding_apr DECIMAL(18,8) NOT NULL DEFAULT 0,
    est_borrow_apr DECIMAL(18,8) NOT NULL DEFAULT 0,
    est_fee_apr DECIMAL(18,8) NOT NULL DEFAULT 0,
    est_net_apr DECIMAL(18,8) NOT NULL DEFAULT 0,
    break_even_days DECIMAL(18,8) NULL,
    eligible TINYINT(1) NOT NULL DEFAULT 0,
    reason VARCHAR(255) NOT NULL DEFAULT '',
    analyzed_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_opportunity (symbol, strategy),
    KEY ix_opportunity_rank (strategy, eligible, score)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS margin_availability (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    symbol VARCHAR(30) NOT NULL,
    asset VARCHAR(20) NOT NULL,
    max_borrowable DECIMAL(30,12) NOT NULL DEFAULT 0,
    avg_daily_interest_rate DECIMAL(20,12) NOT NULL DEFAULT 0,
    est_borrow_apr DECIMAL(18,8) NOT NULL DEFAULT 0,
    est_net_apr DECIMAL(18,8) NOT NULL DEFAULT 0,
    available TINYINT(1) NOT NULL DEFAULT 0,
    checked_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    KEY ix_margin_symbol_time (symbol, checked_at),
    KEY ix_margin_available (available, checked_at)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS positions (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    symbol VARCHAR(30) NOT NULL,
    strategy ENUM('CASH_AND_CARRY','REVERSE_CARRY') NOT NULL,
    status ENUM('OPENING','OPEN','PROTECTING','CLOSING','CLOSED','FAILED') NOT NULL DEFAULT 'OPENING',
    spot_qty DECIMAL(30,12) NOT NULL DEFAULT 0,
    futures_qty DECIMAL(30,12) NOT NULL DEFAULT 0,
    entry_spot_price DECIMAL(30,12) NULL,
    entry_futures_price DECIMAL(30,12) NULL,
    entry_basis_pct DECIMAL(18,8) NULL,
    accumulated_funding_usdt DECIMAL(30,12) NOT NULL DEFAULT 0,
    accumulated_interest_usdt DECIMAL(30,12) NOT NULL DEFAULT 0,
    accumulated_fees_usdt DECIMAL(30,12) NOT NULL DEFAULT 0,
    opened_at DATETIME NULL,
    closed_at DATETIME NULL,
    close_reason VARCHAR(255) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY ix_positions_status (status),
    KEY ix_positions_symbol (symbol, status)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS executions (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    position_id BIGINT UNSIGNED NULL,
    symbol VARCHAR(30) NOT NULL,
    strategy VARCHAR(30) NOT NULL,
    venue VARCHAR(30) NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(30,12) NOT NULL,
    price DECIMAL(30,12) NULL,
    exchange_order_id VARCHAR(100) NULL,
    raw_response JSON NULL,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    KEY ix_exec_symbol_time (symbol, created_at),
    KEY ix_exec_position (position_id)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS risk_events (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    position_id BIGINT UNSIGNED NOT NULL,
    symbol VARCHAR(30) NOT NULL,
    severity ENUM('INFO','WARN','CRITICAL') NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    details VARCHAR(500) NOT NULL,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    KEY ix_risk_position_time (position_id, created_at),
    KEY ix_risk_severity_time (severity, created_at)
) ENGINE=InnoDB;
