# Mainnet Geçiş Kontrol Listesi (1.1) — 18 Eyl 2026

> Testnet→mainnet tek-yön kapıdır. Her madde kutusuz açılış YOK.

## A. Sabitler (tek commit, birlikte — TOKENOMI §kilitli)

- [ ] `BATCH_ODUL_TABAN_MIKRO`: 2000 → 800 mikro
- [ ] `HALVING_BATCH`: 5M → 50B
- [ ] `STRICT_DENETIM` default: `"0"` → `"1"` (öz-denetim toleransı kapanır;
      testnet'te 0 kalır — 3 madenci aynı hostta olduğu için şart)
- [ ] Kuyruk tabanı 5 mikro korunur (B28 kodu zaten kilitli)

## B. Ağ

- [ ] 3+ bağımsız operatör canlı (şu an 0)
- [ ] Dış miner provası (S4) yeşil
- [ ] Tohum-0 dışı 1 düğüm mesh'te

## C. Güvenlik

- [ ] cargo-audit/cargo-deny temiz + SBOM dosyası (DOD #3)
- [ ] `nemes/komut` gossip yolu canlı provası (B33 kodu testte yeşil)
- [ ] Master anahtar töreni (kartta, B24 prosedürü)

## D. Ekonomi

- [ ] B27 tier kararı (gölge-skor 30 günü doldu mu?)
- [ ] Hazine + Enterprise %50 payout mekaniği (TOKENOMI Faz 2)
- [ ] 1 hafta proofsuz izleme (P0-6)

## E. Site/hukuk

- [ ] LICENSE Apache-2.0 sitede yayında (18 Eyl karar + commit; push bekliyor)
- [ ] Terms/privacy mainnet cümleleri (USDT/T+vergi notu operatörde)
- [ ] S7 avukat onayı
