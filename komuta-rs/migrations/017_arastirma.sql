-- 017_arastirma.sql
-- PARANOID-MINER EPIC-04/EPIC-06 (Faz C kuyruk): uzaktan emirle araştırma görevleri.
-- Garanti kuralı: sadece CREATE. Üretim dağıtımına dokunmaz (yeni tablolar + yeni uçlar).
-- Akış: /api/komut{arastir} -> arastirma_gorev(acik) -> miner GET kuyruk -> POST sonuc.

CREATE TABLE IF NOT EXISTS arastirma_gorev (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hedef TEXT NOT NULL,
    parametre TEXT NOT NULL DEFAULT '{}',
    durum TEXT NOT NULL DEFAULT 'acik',
    olusturan TEXT NOT NULL DEFAULT 'komut',
    ts INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_arastirma_gorev_durum ON arastirma_gorev(durum, id);

CREATE TABLE IF NOT EXISTS arastirma_sonuc (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gorev_id INTEGER NOT NULL,
    miner_id TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL DEFAULT '',
    karakter INTEGER NOT NULL DEFAULT 0,
    icerik_hash TEXT NOT NULL DEFAULT '',
    guven REAL NOT NULL DEFAULT 0,
    ts INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_arastirma_sonuc_gorev ON arastirma_sonuc(gorev_id);
