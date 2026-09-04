-- Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
-- 002_batch_spotcheck.sql
-- Batch bazlı ödül + spot-check denetimi.
-- Kanıt başına değil, 20'lik batch tamamlanınca tek ödül.

-- Dagitilan gorevlerin kaydi (batch takibi)
CREATE TABLE IF NOT EXISTS gorevler (
    gorev_id TEXT PRIMARY KEY,
    dagitilan_miner TEXT NOT NULL,
    corpus TEXT NOT NULL,
    offset INTEGER NOT NULL,
    beklenen INTEGER NOT NULL,
    alinan INTEGER DEFAULT 0,
    durum TEXT DEFAULT 'acik',
    odul_mikro INTEGER DEFAULT 0,
    ts INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_gorevler_durum ON gorevler(durum);
CREATE INDEX IF NOT EXISTS idx_gorevler_miner ON gorevler(dagitilan_miner);

-- Kanitlara spot-check bayragi + dogrulama skoru
ALTER TABLE kanitlar ADD COLUMN spot_check INTEGER DEFAULT 0;
ALTER TABLE kanitlar ADD COLUMN dogrulama REAL NULL;

CREATE INDEX IF NOT EXISTS idx_kanitlar_spotcheck ON kanitlar(spot_check);
