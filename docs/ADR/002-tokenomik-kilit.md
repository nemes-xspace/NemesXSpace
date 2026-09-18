# ADR-002: Tokenomik Kilidi + §2b Freni

- Tarih: 08-11 Eyl 2026. Statü: kabul (şifre sahibi onayı).
- Bağlam: sınırsız basım era'yı buharlaştırır (stres tablosu ilk sürümünde
  100 kat hata vardı, 17 Eyl düzeltildi).
- Karar: tavan 210M, era1 800 mikro, HALVING 50B; günlük dağıtım freni
  (≤HALVING/1460) era-1'i ~4 yıl korur; madenci-başı tavan sybil freni.
  Testnet cömert değerlerle sürer (2000 mikro + 5M), mainnet tek commit.
- Alternatifler: API-hakkı modeli — elendi 08 Eyl (ödül netliği).
- Sonuçlar: emisyon 1M'da bile korunur; verim 1/N seyrelir (bilinen kabul).
- Geri-dönüş: sabitler mainnet lansman commit'inde tek noktadan değişir.
