# NEMES-Free Damıtma Planı — Teknik
**Tarih:** 28 Ağustos 2026
**Öğretmen:** NEMES-Sovereign (%100, 70B+, 40M vektör index'li, henüz eğitiliyor — gut_en sonrası)
**Öğrenciler:** 3B (default) ve 30B (high) — aynı hattan, farklı kapasite

---

## 1) Yöntem — White-box + Black-box Hibrit

- **White-box (logit):** Öğretmenin son katman logit'lerini KL-divergence ile öğrenciye öğret (hızlı, stabil). Sıcaklık T=2.0
- **Black-box (response):** Öğretmenin ürettiği 5M sentetik instruction-response'u SFT olarak ekle
- **Karışım:** Loss = 0.6 × KL + 0.4 × SFT

## 2) Veri Hattı (BEYİN'den)

```
Sovereign Index (40M vektör)
    ↓ FAISS retrieval ile 5M sentetik prompt üret (self-instruct, Türkçe dahil)
BEYİN Korpusları (22 corpus, 109M doküman)
    ↓ 4500 MAX_CHAR ile chunk'la, 3B için 2k, 30B için 4k bağlam
Karışım: %50 sentetik + %30 CC100/Wiki + %20 kod/mat (arXiv, gut_en)
```

- **1B Nano:** 2k bağlam, batch 1024, lr 2e-5, 8×A100 2 gün
- **3B Default:** 2k bağlam, batch 1024, lr 2e-5, 8×A100 3 gün
- **8B Pro:** 3k bağlam, batch 768, lr 1.5e-5, 8×A100 5 gün
- **30B High:** 4k bağlam, batch 512, lr 1e-5, 16×A100 7 gün (veya 2×1080Ti ile sıralı 4 hafta)

## 3) Stabilite Önlemleri

- **En stabil baz:** Llama-3.2-3B ve Qwen2.5-32B (seçildi)
- **Checkpoint her 500 step**, eval her 1000 step — düşerse geri sar
- **Quantization-aware:** Damıtmadan sonra Q4_K_M GGUF'a çevir, telefonda test et (25 tok/s altı kabul edilmez)

## 4) Değerlendirme

Her 1000 step'te: MMLU, GSM8K, HumanEval, Türkçe MMLU. Hedef: yukarıdaki tabloda %15-20 fark.

## 5) Çıktılar

- `NemesXSpace/NEMES-Free-3B` (GGUF + HF)
- `NemesXSpace/NEMES-Free-30B` (GGUF + HF)
- Her ikisi de `model/` altinda, Tescilli — Tum Haklari Saklidir

## 6) Sonraki Adım

1. Sovereign'in gut_en sonrası checkpoint'ini bekle (1-2 gün)
2. 3B damıtmayı başlat (küçük, hızlı geri bildirim)
3. 3B stabil ise 30B'a geç

*Not: Bu plan `00-PROJE-TANIMI.md` ve `MODEL-KARTI-Free.md` ile senkron tutulur.*
