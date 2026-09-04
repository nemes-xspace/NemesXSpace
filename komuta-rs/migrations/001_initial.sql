-- Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
-- 001_initial.sql
-- NEMES Komuta veritabanı şeması

-- Madenciler tablosu
CREATE TABLE IF NOT EXISTS miners (
    miner_id TEXT PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,
    cuzdan TEXT NOT NULL,
    makine_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_seen INTEGER DEFAULT 0,
    coin_mikro INTEGER DEFAULT 0,
    pay INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_miners_token ON miners(token);
CREATE INDEX IF NOT EXISTS idx_miners_last_seen ON miners(last_seen);

-- Görev cursor tablosu (hangi maddeye kadar dağıtıldı)
CREATE TABLE IF NOT EXISTS gorev_cursor (
    corpus TEXT PRIMARY KEY,
    son_id INTEGER DEFAULT 0
);

-- Kanıtlar tablosu
CREATE TABLE IF NOT EXISTS kanitlar (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gorev_id TEXT NOT NULL,
    madde_id INTEGER NOT NULL,
    miner_id TEXT NOT NULL,
    v_int8_b64 TEXT NOT NULL,
    v_min REAL NOT NULL,
    v_max REAL NOT NULL,
    odul_mikro INTEGER NOT NULL,
    ts INTEGER NOT NULL,
    UNIQUE(gorev_id, madde_id)
);

CREATE INDEX IF NOT EXISTS idx_kanitlar_miner ON kanitlar(miner_id);
CREATE INDEX IF NOT EXISTS idx_kanitlar_gorev ON kanitlar(gorev_id);
CREATE INDEX IF NOT EXISTS idx_kanitlar_ts ON kanitlar(ts);

-- Coin defteri (append-only)
CREATE TABLE IF NOT EXISTS ledger (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    miner_id TEXT NOT NULL,
    delta_mikro INTEGER NOT NULL,
    neden TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    ts INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ledger_miner ON ledger(miner_id);
CREATE INDEX IF NOT EXISTS idx_ledger_ts ON ledger(ts);
CREATE INDEX IF NOT EXISTS idx_ledger_epoch ON ledger(epoch);

-- İmza kayıtları (görev bazlı, opsiyonel)
CREATE TABLE IF NOT EXISTS imzalar (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gorev_id TEXT NOT NULL,
    miner_id TEXT NOT NULL,
    imza_b64 TEXT NOT NULL,
    pubkey_b64 TEXT NOT NULL,
    ts INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_imzalar_gorev ON imzalar(gorev_id);