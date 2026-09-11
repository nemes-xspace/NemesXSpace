-- 011_guvenlik.sql
-- Guvenlik mimarisi (docs/GUVENLIK-MIMARISI.md):
-- 1) kor_esleme: miner'a giden kor (rastgele) ID <-> gercek madde_id eslemesi.
--    Dagitim basina uretilir, kanit/denetim donusunde cozulur.
-- 2) kanaryalar + kanarya_dagitim: sizinti tuzak metinleri ve dagitim izi.
-- 3) kara_liste: kanitli hainlikte imha (ban + pay sifirlama izi).

CREATE TABLE IF NOT EXISTS kor_esleme (
    kor_id INTEGER PRIMARY KEY,
    madde_id INTEGER NOT NULL,
    gorev_id TEXT NOT NULL,
    ts INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_kor_esleme_gorev ON kor_esleme(gorev_id);

CREATE TABLE IF NOT EXISTS kanaryalar (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metin TEXT NOT NULL UNIQUE,
    aktif INTEGER DEFAULT 1,
    ts INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS kanarya_dagitim (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kanarya_id INTEGER NOT NULL,
    gorev_id TEXT NOT NULL,
    miner_id TEXT NOT NULL,
    ts INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_kanarya_dagitim_miner ON kanarya_dagitim(miner_id);

CREATE TABLE IF NOT EXISTS kara_liste (
    miner_id TEXT PRIMARY KEY,
    neden TEXT NOT NULL,
    ts INTEGER NOT NULL
);

-- Baslangic kanaryalari: disarida indexlenmemis sentetik cumleler.
-- dagitimda %4 olasilikla gorevlere serpistirilir (madde_id = -kanarya_id).
INSERT OR IGNORE INTO kanaryalar (id, metin, aktif, ts) VALUES
(1, 'Kırıkhan ovasında mor salkımlı akasya rüzgârla fısıldaşırken topraktaki çakıllar sayılır.', 1, 1788000000),
(2, 'Lacivert bakraçtaki yoğurt mayası tutmayınca köylü teyze tarif defterine yıldız çizdi.', 1, 1788000000),
(3, 'Paslı teneke kutunun içindeki cam bilyeler yağmur sesiyle şıngırdarken kedi ürktü.', 1, 1788000000),
(4, 'Saman balyalarının arasında uyuyan kirpi, traktör sesini duyunca dikenlerini kabarttı.', 1, 1788000000),
(5, 'Eski değirmenin su oluğunda biriken yosunlar, değirmencinin torununa masal anlattı.', 1, 1788000000);
