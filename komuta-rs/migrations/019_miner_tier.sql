-- 019_miner_tier.sql
-- B27 gölge-skor: tier SADECE görüntülenir, ödemeye dokunmaz.
CREATE TABLE IF NOT EXISTS miner_tier (
    miner_id TEXT PRIMARY KEY,
    v30 INTEGER NOT NULL DEFAULT 0,
    stake_mikro INTEGER NOT NULL DEFAULT 0,
    itibar INTEGER NOT NULL DEFAULT 100,
    sure_gun REAL NOT NULL DEFAULT 0,
    ham REAL NOT NULL DEFAULT 0,
    tier INTEGER NOT NULL DEFAULT 1,
    ts INTEGER NOT NULL
);
