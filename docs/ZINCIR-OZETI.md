# Zincir Yönetici Özeti — 7 Prompt Kapanışı (19 Eyl 2026)

## En kritik 5 bulgu (zincir-boyu)

1. **Deploy-boşluğu (RT-001):** kod güvenli, canlı korumasız. Tek komutla kapanır (systemd düzelince). Etki: KRİTİK-şu-an.
2. **Tek-insan (bus=1):** bilgi, kart, karar, müdahale tek elde. Etki: KRİTİK-yapısal.
3. **SPOF tek-host + yedek-tek-lokasyon:** yangın/hırsızlık = mutlak kayıp riski. Etki: KRİTİK.
4. **Token düz-metin + auth-körlük:** kopyala-taklit et, iz bırakma. Etki: YÜKSEK.
5. **Ekonomi doğrulanmadı:** mekanik sağlam, payın değeri umut. Etki: YÜKSEK (itibar).

## En zayıf 3 kurtarma

Host-kaybı (plan yok), ekip-kaybı (plan yok), dış-duyuru kanalı (yok).
Kanıtlı güçlüler: DB-restore (RTO~3dk), binary-rollback, yedek-doğrulama.

## En acil 3 eylem (sahipli)

1. Deploy (sistemd düzelir düzelmez) — sahip: operatör.
2. Vasiyet-mektubu + yedek-kart + uzak-yedek — sahip: hak-sahibi, 2 hafta.
3. B39 token-hash + IP-logging-canlı + kurtarma-prosedürü — sahip: denetçi+sahip, 1 hafta.

## Döngü takvimi (sahipli)

| Periyot | İş | Sahip | Kanıt |
|---|---|---|---|
| Günlük | Karşı-istihbarat bakışı + defter `dogrula.py` | operatör (izleme.py yanına) | log + SAGLAM |
| Haftalık | Kırmızı mini-tatbikat (1 senaryo) | denetçi | IR kaydı |
| Aylık | Mimari gözden-geçirme + iz-küçültme + restore-provası | denetçi | RR kaydı |
| 3 aylık | Felaket provası + tam kırmızı tatbikat | sahip+denetçi | FELAKET-PROVASI.md |
| Yıllık | Kapsamlı denetim + süreklilik + dış-göz | sahip | denetim raporu |

## Nihai kural (politika)

Kanıt yoksa güven yok. Kayıt yoksa olay yok. Tatbikat yoksa plan yok.
İz kontrolü yoksa gizlilik yok. Alternatif yoksa süreklilik yok.

## Zincir durumu

P1 keşif ✅ · P2 mimari ✅ · P3 kırmızı ✅ · P4 defter ✅ (47 kayıt,
SAGLAM) · P5 felaket ✅ (restore kanıtlı) · P6 süreklilik ✅ (ölçülü) ·
P7 karşı-istihbarat ✅ (supurme temiz). Döngü 2. tura hazır.
