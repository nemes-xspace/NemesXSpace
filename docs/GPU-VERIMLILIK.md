# GPU Verimlilik Raporu — Ölçülü Sonuç (19 Eyl 2026)

> Soru: "GPU %0, %100 yap." Cevap (ölçülü): %100 yanlış hedef;
> doğru hedef kanıt/sn + kanıt/watt. Gerekçe aşağıda.

## Ölçümler

- Donanım: GTX 1080 Ti 11G; boşta %0-6, 66-68W/250W.
- Embed gecikmesi: sessiz-sunucu 3.4sn/8-metin, yüklü 6-20sn
  (kuyruk + bilinen llama-yavaşlaması: ~2saatte 4x, rolling-restart yönetir).
- Görev-büyüklüğü: 20 metin × ~1000 token (komuta TASK_BATCH=20).
- A/B (miner-b isci 1→4, 10dk): 1461 → 1546 (+%6, gürültü-sınırında).
  Karar: GERİ ALINDI (sadelik; kazanç yok).

## Neden %100 olmaz (matematik)

- İş patlamalı: HTTP-gidiş/geliş + DB + denetim aralarında GPU BOŞTA
  bekler. Ortalama-%100 için boru hattının 20x büyümesi gerekir;
  komuta-dağıtım ve denetim-kapasitesi tavan (birikim büyüyor).
- %100 = ısı+gürültü+elektrik, karşılıksız. Verimlilik = kanıt/watt.

## Ayar tablosu (cmd'den — mevcut bayraklar)

| Bayrak | Varsayılan | Ne zaman artır |
|---|---|---|
| `--isci N` | 1 | Gömme-sunucusu boşta + komuta kuyruğu doluysa (2-4) |
| `--kira-adet N` | 2000 | Kopma-sık ise büyüt (daha-az HTTP); takılma varsa küçült |
| `--shard-adet N` | 2000 | Depolama-taahhüdüne göre |
| `--embed-api URL` | 1241 | En-boş sunucuya yönlendir (1241/1242/1251) |

## Gerçek kollar (sahipli)

1. llama-yavaşlama döngüsü (bekçi yönetiyor) — kalıcı-çözüm: sürüm-yükseltme.
2. Sunucu-birleştirme deneyi (3 proses → 1-2): riskli (tek-nokta), tatbikat-ister.
3. Komuta-dağıtım hızı: tavan buradaysa GPU ayarları etkisiz (ölçülecek).
