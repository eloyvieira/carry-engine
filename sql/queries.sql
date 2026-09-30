-- =====================================================
-- 1. TOP 20 OPPORTUNITIES
-- =====================================================

SELECT
    symbol,
    strategy,
    est_funding_apr,
    est_borrow_apr,
    est_net_apr,
    basis_avg_1d_pct,
    basis_std_7d_pct,
    score,
    eligible
FROM opportunities
WHERE eligible = 1
ORDER BY est_net_apr DESC
LIMIT 20;


-- =====================================================
-- 2. TOP REVERSE CARRY
-- =====================================================

SELECT
    symbol,
    est_funding_apr,
    est_borrow_apr,
    est_net_apr,
    basis_avg_1d_pct,
    basis_std_7d_pct,
    score
FROM opportunities
WHERE strategy = 'REVERSE_CARRY'
  AND eligible = 1
ORDER BY est_net_apr DESC
LIMIT 20;


-- =====================================================
-- 3. TOP CASH AND CARRY
-- =====================================================

SELECT
    symbol,
    est_funding_apr,
    est_borrow_apr,
    est_net_apr,
    basis_avg_1d_pct,
    basis_std_7d_pct,
    score
FROM opportunities
WHERE strategy = 'CASH_AND_CARRY'
  AND eligible = 1
ORDER BY est_net_apr DESC
LIMIT 20;


-- =====================================================
-- 4. OPPORTUNITIES WITH NEGATIVE BASIS
-- =====================================================

SELECT
    symbol,
    strategy,
    est_funding_apr,
    est_borrow_apr,
    est_net_apr,
    basis_avg_1d_pct,
    basis_std_7d_pct,
    score
FROM opportunities
WHERE eligible = 1
  AND basis_avg_1d_pct < 0
ORDER BY est_net_apr DESC
LIMIT 20;


-- =====================================================
-- 5. MOST STABLE BASIS
-- =====================================================

SELECT
    symbol,
    strategy,
    est_net_apr,
    basis_avg_1d_pct,
    basis_std_7d_pct,
    score
FROM opportunities
WHERE eligible = 1
ORDER BY basis_std_7d_pct ASC
LIMIT 20;