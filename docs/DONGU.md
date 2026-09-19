# Sürekli İyileştirme Döngüsü — Prompt 18 (19 Eyl 2026)

> Tüzük: döngü kararını AI vermez (sahip verir); her tur kanıtlı+kayıtlı;
> test atlatılamaz; kapsam daraltılamaz (kritik).

## Mimari (çalışan)

- Günlük-otomatik: `scripts/dongu-gunluk.py` (salt-okunur: defter-zincir,
  site-CI, `/health`) → `nemes-dongu.timer` (günlük). Tur-1 koşuldu: 0 bulgu.
- Haftalık/aylık/3-aylık/yıllık: sahip-takviminde (bu dosya + ZINCIR-OZETI).
- Tetikleyiciler: zaman (timer) + olay (alarm/IR → olağanüstü-tur).
- Kayıt: tur-çıktısı + defter RR.

## Otomasyon sınırları (AI)

Yapabilir: toplama, ön-analiz, rapor-taslağı, bu betik. Yapamaz: döngü-kararı,
onaysız-yama, geri-alma, otorite/halef/şifre işleri, onaysız-dış-bağlantı.

## Tur-1 kaydı (19 Eyl)

Kapsam: zincir+CI+sağlık. Sonuç: 0 bulgu. Sonraki: günlük-otomasyon izler;
ilk haftalık-gözden-geçirme sahipte.

## Göstergeler (hedef)

Tur-tamamlama %100 · doğrulama %>95 · regresyon <%5 · yeni-açık 0 ·
AI-ihlal 0 · kayıt %100.
