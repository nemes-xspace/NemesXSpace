-- 013_spot_salt.sql
-- B-4: spot-check orneklemesi gunluk salt ile karistirilir.
-- blake3(gorev_id + madde_id + salt) ilk bayt % 100 < SPOT_CHECK_YUZDE.
-- Salt gunluk uretilir (get-or-create), DB'de durur: ayni gun icinde
-- kararlar tekrarlanabilir, gunler arasi ongorulemez (seckinci durustluk engeli).
-- Garanti kurali 3 uyumu: sadece CREATE (ALTER/DROP yok).

CREATE TABLE IF NOT EXISTS spot_salt (
    gun INTEGER PRIMARY KEY,
    salt BLOB NOT NULL,
    ts INTEGER NOT NULL
);
