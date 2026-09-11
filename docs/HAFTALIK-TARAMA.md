# Haftalık Sık Dokuma — Kendini Denetleme Ritüeli (v1.0, Eyl 2026)
> Her hafta aynı gün (öneri: Pazartesi), aynı liste, 1 saat.
> Amaç: amatörlüğü sistemle yenmek. Profesyonellik = tekrarlanan titizliktir.

## 1. Sayılar (15 dk) — siteyle defter tutuyor mu?
- [ ] `SELECT COUNT(*) FROM kanitlar` + `SUM(coin_mikro) FROM miners` → site rakamlarıyla karşılaştır, fark varsa siteyi güncelle
- [ ] FAISS envanteri (`*.faiss` boyut + ntotal örneklemesi) → verify sayfası
- [ ] Disk/ısı trendi (daralma/varsa alarm eşiği %90)

## 2. Güvenlik (15 dk)
- [ ] kara_liste + strike listesi boş mu? (doluysa nedenleri gözden geçir)
- [ ] kanarya_dagitim sayımı (dağıtıldı mı? dış tarama ipucu var mı?)
- [ ] log'larda düz metin sızıntısı tara (`grep -ri "ozet\|metin" loglar`)
- [ ] Yedek tazeliği (son komuta-backup < 26 saat mi?)

## 3. Site (15 dk) — amatörlük taraması
- [ ] Kırık link (tüm sayfalar + indirme zip'i indirilebilir mi?)
- [ ] Bayat sayı (tarih damgaları güncel mi?)
- [ ] 8 dilde boş/eksik anahtar tara (EN fallback'e düşen var mı?)
- [ ] Mobil görünüm (1 sayfa spot check)

## 4. Topluluk + itibar (10 dk)
- [ ] Açık issue/mention var mı (X/Reddit/Discord)? 24 saatte cevap kuralı
- [ ] İlk payout'a kaç hafta kaldı (H takibi)?

## 5. Öğrenme (5 dk) — yeni şey keşfi
- [ ] Haftanın 1 makalesi: embedding/RAG/P2P alanından 1 paper oku, 3 cümlelik notu buraya ekle
- [ ] Rakip taraması: 1 proje (neden iyiler, ne çalınır?)

---
*İlk tarama: 15 Eyl 2026 Pazartesi. Liste yaşar — eksik madde eklenir.*
