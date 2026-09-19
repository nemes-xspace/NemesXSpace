# RUNBOOK: Komuta Rollback (DB + binary birlikte donus)

> Son prova: 18 Eyl 2026 (ispatli bulgular). Hedef sure: <10dk kesinti.

## KRITIK BULGU (18 Eyl provasi, kopya DB'de ispatli)

`.bak` binary TEK BASINA geri alinamaz: migration uygulanmis DB'de eski
binary `migration N was previously applied but is missing` hatasiyla ACILMAZ
(sqlx kati davranir). Yani rollback = **YEDEK DB + eslesen binary** birlikte.
`.bak` zinciri yalnizca ayni-migration nesli icinde ise yarar.

## Yedek gecerliligi (19 Eyl dogrulandi)

`yedek/komuta-2026-09-19-0321.db`: 4.763.510 kanit ile tutarli (8,1GiB,
`integrity_check=ok`, freelist-temiz kopya, kesik degil). Gece yedekleri
SAGLAM. Not: 00:38 timer calismasi pages=100 yuzunden 2,7sa takilmisti;
script pages=-1 + tmp+rename + dogrulama + rotasyona cekildi (19 Eyl).

## Ne zaman

- Yeni deploy sonrasi hata/performans gerilemesi (5dk gozlem esigi).
- `journalctl -u nemes-komuta -p err` artisi veya `/health` 3x ust uste yanitsiz.

## Geri alma (HER ADIMDA DOGRULA)

1. Hedef surumu sec: yedek DB tarihi + O TARIHTEKI binary (`.bak` listesi).
   Emin degilsen ONCE kopya DB'de dene (`/tmp/rb-test.db` + baska port).
2. Miner'lari durdur (a/b sirali; win uretir, kaybi supurme kapatir) +
   `sudo -n systemctl stop nemes-komuta`.
3. Canli DB'yi yana al (SILME): `cp komuta.db komuta.db.<tarih>-once.db`.
4. Yedegi yerine koy: `cp yedek/<secilen>.db komuta.db` + eslesen binary'yi
   `bin/` altina kopyala (`cp bin/komuta-rs.<tarih>.bak bin/komuta-rs`;
   ayni sekilde `bin/nemes-miner.<tarih>.bak`).
5. Baslat: `sudo -n systemctl start nemes-komuta`; `is-active` + `/health` + havuz (~2dk).
6. Miner'lari baslat (`sudo -n systemctl start nemes-miner-a nemes-miner-b`), `Batch tamam` + RSS izle (10dk).
7. Kaydet: ALTYAPI-KAYIT §4'e satir + neden.

## Bilinen tuzaklar

- `cp` "Text file busy" = servis ayakta (stop'u dogrula).
- D-Bus takilirsa `sudo -n systemctl` (polkit helper asili kalir).
- Miner retry (BUG-2): 30sn-2dk bosluk normaldir.
- Asla `.bak` silme (en az 3 nesil) + canli DB'yi yedeksiz ezme.
- `pkill -f` desenin komut satirinda gecmesin (kendi shell'in + canli olur).
