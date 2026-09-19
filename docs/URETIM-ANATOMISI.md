# Madenci Üretim Anatomisi — Canlı Otossi (19 Eyl 2026, ~14:30)

> Ölçüm penceresi: son 1 saat. Yöntem: canlı DB salt-okunur sorgular +
> wiki + FAISS dosya-incelemesi. Varsayım yok.

## 1. Ne üretiliyor (özet)

- 26.531 kanıt/saat, %100 `newscrawl_tr` (Türkçe haber-crawl).
- Örnek girdi: eğitim-semineri haberi (Maslak, 1200 eğitimci) — kamusal metin.
- Üç madenci: 523d3c47 (15.5K/sa) + a41b0d81 (9.7K) + fa8edb50/VM (2K).

## 2. Tek-kanıt anatomisi (ölçülü örnek: id 5238699)

| Alan | Değer | Anlam |
|---|---|---|
| `gorev_id` | newscrawl_tr:4307746:20:kkira:1789827979:b9bb061e03e4-26 | korpus:offset:20'li-batch:kira:epoch:hash-sıra |
| `madde_id` | 4307765 | wiki satır-kimliği (kamusal) |
| `v_int8_b64` | 1024 karakter | 768-boyut int8 vektör (768 bayt → b64) |
| `v_min/v_max` | -0.1655 / +0.1117 | dekuantizasyon aralığı (geri-çevrilebilir) |
| `odul_mikro` | 0 | kanıt-başı ödeme YOK (ödeme batch-kapanışta) |
| `spot_check` | 0/1 | %11,9 bayrak (hedef %10 + dinamik) |
| `dogrulama` | NULL→0.99 | denetimde kosinüs yazılır (eşik 0.98) |

## 3. Kalite (ölçülü)

- Doğrulanmış toplam: 380.632; son-örneklem kosinüsleri ~0.99 (eşiğin üstünde).
- Denetim FIFO: bugünkü denetimler ~50-saatlik eski kanıtlara düşer
  (taze kanıt sırasını bekler — tasarım, gecikme-değil).
- Spot oranı %11,9 (dinamik-artırımlı).

## 4. Ödeme anatomisi (son 1 saat)

- Batch-kapanış: 20 kanıt = 2000 mikro (0.002 NEMES), `odeme_yaz` atomik.
- Dağılım: 1.34 + 0.84 + 0.17 NEMES (üretim-orantılı).
- Kanıt-satırı ödülü: 0 (muhasebe batch'te; toz birikir).

## 5. Vektörler nereye gidiyor (19 Eyl KAPANDI ✅ + sadakat-notu)

- **DB'de birikir** (4.3M satır, 9.2G): dayanıklı, sorgulanabilir, denetlenebilir.
- **FAISS'e AKIYOR** (19 Eyl 18:36): `wiki_maden_tr.faiss` kuruldu —
  382.687 madenci-vektörü, 18MB, nlist-512/PQ32, ntotal-doğrulamalı.
  Haftalık otomasyon devrede (`nemes-faiss.timer`, Pazar 03:00).
- **Sadakat-notu (ölçülü):** kaba-kuvvet self-rank=1 (veri mükemmel),
  index recall@10 = 2-3/10 (PQ32 + sıkışık-haber-korpusu; semantik-arama
  için yeterli, hassas-eşleşme için zayıf). İyileştirme yolu: PQ64/OPQ/
  nprobe-yükseltme (görev, acil-değil).

## 6. Madenci profilleri (son 1 saat)

| Madenci | Kanıt | Ödeme | Rol |
|---|---|---|---|
| 523d3c47 (host-A, 4 işçi) | 14.798 | 1.34 | hacim-lideri |
| a41b0d81 (host-B) | 9.745 | 0.84 | hacim-ikincisi |
| fa8edb50 (VM-Windows) | 1.986 | 0.17 | düşük-hız + 72 denetim/10dk (doğrulayıcı-ağır) |
