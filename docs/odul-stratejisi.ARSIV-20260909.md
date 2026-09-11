# NEMES-X Ödül Stratejisi — 2 Aşama
**Karar Tarihi:** 28 Ağustos 2026
**Karar Veren:** d3str0y1ng
**Durum:** Onaylandı — 2. seçenek ile başla, sonra 1. seçeneğe geç

---

## Faz 1 — Bootstrap (Madenci Kazanana Kadar) → **Seçenek 2**

**Madenci = Müşteri Döngüsü**
- Madencinin kazandığı her pay, **nakit değil API hakkı** olarak verilir
- Örn: 1000 pay = Pro'da 10.000 sorgu hakkı
- Dışarıdan para girmeden, madenci kendi kullanacağı AI hakkını kazanmış olur
- Nakit çıkışı **sıfır**, hazineye dokunulmaz
- Amaç: Ağ büyüyene kadar hazineyi korumak, madenciyi içeride tutmak

**Ne zaman biter?** İlk Enterprise geliri gelip `H = gelir × %50` madenciye nakit dağıtmaya yettiğinde.

---

## Faz 2 — Sürdürülebilir (Madenci Kazandıktan Sonra) → **Seçenek 1**

**Hazine + Gelir Paylaşımı**
- Haftalık Havuz `H = max(Enterprise_geliri × %50, Treasury_haftalık)`
- İlk 6 ay Treasury'den enflasyonist ödül (örn. 100M NEMES genesis fonu, haftalık dilimlerle)
- 6 ay sonunda Treasury tükenir, sistem tamamen `gelir × %50` ile kendi kendini besler
- Pay Değeri `P = H / toplam_pay`, Ödeme `pay × P` (nakit)

---

## Neden Bu Sıra?

1. Başta hazineyi yakmadan ağı büyüt (2. seçenek risksiz)
2. Madenci zaten API hakkını kullanacağı için sadık kalır
3. Enterprise geliri gelince nakde geçmek güven verir ("artık gerçek para kazanıyorsun")

*Kaynak: 28 Ağu sohbet kararı — NemesXSpace/README ve 00-PROJE-TANIMI ile senkron.*

---
## ARŞİV NOTU (09 Eyl 2026)
Bu belge `TOKENOMI.md v1.0-KİLİTLİ` ile yürürlükten kalkmıştır (R1 coin+halving
kararı; Faz 1 API-hakkı ifadesi geçersizdir). Tarihçe için saklanır.
