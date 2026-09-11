-- 009_onarim.sql
-- Onarim/çoğaltma takibi: kimin hangi parçayı kime taşıdığı.
-- Bayt transferi v1'de komuta relay üzerinden (C7b'de miner-to-miner).

CREATE TABLE IF NOT EXISTS onarimlar (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parca_hash TEXT NOT NULL,
    hedef_miner TEXT NOT NULL,
    durum TEXT DEFAULT 'acik',
    ts INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_onarim_hedef ON onarimlar(hedef_miner, durum);
CREATE INDEX IF NOT EXISTS idx_onarim_parca ON onarimlar(parca_hash, durum);
