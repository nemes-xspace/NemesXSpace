# Kurucu Anahtarı Yol Haritası — TASLAK (onay bekliyor)

> Durum: taslak 16 Eyl 2026. Dış denetim (§9.1) "kurucu anahtarı compromise = catastrophic" diyor — doğru. Bu belge kademeli çıkış planıdır.

## Mevcut durum (gerçek)
- Tek Ed25519 master anahtar; `MASTER_PUBKEY_B64` komuta-rs'ye gömülü, `MASTER_PUBKEY` nemes-core'da.
- İmza kullanımı: `komut` endpoint'i (imha/affet dahil), shard ilan TOFU bağı.
- Üretim: `nemes-p2p/offline-ceremony/` scriptleri (çevrimdışı üret, şifreli sakla).
- Miner kanıtları imzasız (Faz-2 işi olarak kayıtlı).

## Hedef mimari (3 kademe)
1. **K1 (şimdi–Kas 2026):** Prosedür sertleştirme — anahtar çevrimdışı makinede, imzalama hava-boşluklu akışla (`offline-ceremony` + QR/taşıma), her imza çift-kayıt (tarih+sebep+komut hash'i). Operatör yedeği mühürlü zarfta ayrı mekânda.
2. **K2 (Kas–Oca, genesis öncesi):** Eşik imza hazırlığı — 2/3 (kurucu + 2 emanetçi) veya 3/5 treasury multi-sig ile aynı tören. Kod tarafı: `verify_signed_command` çok-anahtarlı doğrulamaya genişletilir (eşik + key-id).
3. **K3 (2027):** Yetki devri — epoch imzalama yetkisi seçilmiş operatör kuruluna taşınır; kurucu anahtarı acil-durum (recovery) anahtarına indirgenir, kullanımı şeffaf duyuruya bağlanır.

## Kabul kriterleri
- K1: yazılı prosedür + ilk çift-kayıtlı imza provası.
- K2: genesis duyurusu 3 imzalı + kilit adresleri zincirde doğrulanabilir.
- K3: kurucu imzası olmadan 1 tam epoch kapanışı.

## Açık kararlar (operatör)
- Emanetçiler kim? (güven + erişilebilirlik)
- Eşik kaç? (öneri: 2/3 K2, 3/5 treasury)
- Acil-durum anahtarının saklama yeri?
