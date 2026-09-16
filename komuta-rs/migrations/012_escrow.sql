-- 012_escrow.sql
-- B-1: supheli kanitlarin batch payi emanette tutulur.
-- Kapanista (spot_check=1 AND dogrulama IS NULL) olan her kanit icin payi
-- buraya yazilir; denetim GECERSE odenir (ledger 'escrow'), KALIRSA silinir
-- (yanar, odenmemis oldugu icin ledger'e dokunulmaz).
-- Garanti kurali 3 uyumu: sadece CREATE (ALTER/DROP yok).

CREATE TABLE IF NOT EXISTS escrow (
    gorev_id TEXT NOT NULL,
    madde_id INTEGER NOT NULL,
    miner_id TEXT NOT NULL,
    miktar_mikro INTEGER NOT NULL,
    ts INTEGER NOT NULL,
    PRIMARY KEY (gorev_id, madde_id)
);
CREATE INDEX IF NOT EXISTS idx_escrow_miner ON escrow(miner_id);
