-- 014_teklifler.sql
-- B21: bosluk teklifleri (kendi-kendine ogrenme kapisi). Madenci kapsama
-- eksigi araligi onerir (stake'li); supurme oncelikle orayi dagitir;
-- kapanista bulucu payi + kapsama bitince iade; vadede iade.
-- Garanti kurali 3 uyumu: sadece CREATE (ALTER/DROP yok).

CREATE TABLE IF NOT EXISTS teklifler (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    miner_id TEXT NOT NULL,
    corpus TEXT NOT NULL,
    baslangic INTEGER NOT NULL,
    bitis INTEGER NOT NULL,
    stake_mikro INTEGER NOT NULL,
    durum TEXT NOT NULL DEFAULT 'acik',
    ts INTEGER NOT NULL,
    kapanma_ts INTEGER DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_teklifler_durum ON teklifler(durum, ts);
CREATE INDEX IF NOT EXISTS idx_teklifler_aralik ON teklifler(corpus, baslangic, bitis);
