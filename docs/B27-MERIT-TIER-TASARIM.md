# B27 Merit-Tier Tasarımı — TASARIM v1 (taslak, 18 Eyl 2026)

> Sorun: ödül flat (`S=V`). Çiftlik beklentisi (S4) farklılaşma istiyor ama
> self-report donanım (B23 `miner_yetenek` beyanı) oyunlanabilir.
> İlke: tier SADECE doğrulanmış zincir-içi veriden türetilir, beyandan asla.

## 1. Girdiler (hepsi zincir-içi, oyunlanması maliyetli)

- `V30`: son 30 gün doğrulanmış vektör (kanitlar.dogrulama IS NOT NULL, gecerli).
- `STAKE`: kilitli stake (mikro). Kilit vadelidir (erken-çıkış yanığı).
- `ITIBAR`: mevcut itibar (strike/slash geçmişi).
- `SURE`: kesintisiz görülen gün (last_seen sürekliliği).

## 2. Tier formülü (öneri, simülasyon şart)

```
ham = V30_norm × 0.5 + STAKE_norm × 0.3 + ITIBAR_norm × 0.1 + SURE_norm × 0.1
tier = 1 + floor(ham × 4)          # T1..T5
carpan = [1.0, 1.25, 1.5, 2.0, 3.0][tier-1]
S = V × carpan(tier)               # MADENCI-REHBERI'ne geri doner (B27 ile)
```

- Normalizasyon: ağ-medyanına göre (sabit eşik yok — ağ büyüdükçe ölçeklenir).
- Tavan: T5 carpan 3.0 (balina-farkı sınırlı; merkezsizlik korunur).
- Sybil: tier başına STAKE eşiği (bölünerek çoklanmak kârsızlaşır).
- Düşüş: tier 7 günde bir yeniden hesaplanır (kötü hafta düşürür).

## 3. Kötüye-kullanım senaryoları

- Sahte stake: kilit + yanık → maliyeti getiriyi aşar (parametre simülasyonu ŞART).
- Vektör şişirme: denetim + slash aynen çalışır (tier, doğrulamayı gevşetmez).
- Eski-şöhret: SURE ağırlığı düşük (0.1), V30 tazeliği zorunlu.

## 4. Geçiş planı

1. Testnet: tier HESAPLA ama ÖDEME (gölge-skor, `miner_tier` tablosu salt-görüntü).
2. 30 gün gölge-veri → parametre ayarı → duyuru.
3. Mainnet açılışında aktif (TOKENOMI revizyonu gerektirir — kilitli belge, imza şart).

## 5. Açık sorular (operatör)

- Carpan tavanı 3.0 mı (çiftlikler için yeterli mi)?
- STAKE eşiği ne (Sybil maliyeti simülasyonu)?
- V30 penceresi 30 gün mü (mevsimsellik)?
- Tier singles-makine küçük madenciyi ezer mi (T1 taban koruması)?
