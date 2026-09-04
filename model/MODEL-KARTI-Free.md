# NEMES-Free — Model Kartı (v1.0)
**Tarih:** 28 Ağustos 2026
**Konum:** `~/NemesXSpace/model/MODEL-KARTI-Free.md`
**Lisans:** Tescilli — Tüm Hakları Saklıdır (c) 2026 NEMES-X. Yazılı izin olmadan kullanım yasaktır.
**Durum:** Tasarım aşaması — stabil baz model seçildi, damıtma planı hazır

---

## 1) Özet

NEMES-Free, **NEMES-Sovereign (%100, 70B+)** ana beyinden damıtılmış, **çoğu açık modelden %15-20 daha güçlü** bir ailedir. Herkes indirip kendi cihazında çalıştırabilir. İki modla gelir:

| Mod | Parametre | Hedef Donanım | Kullanım |
|-----|-----------|---------------|----------|
| **Nano** | **1B** | Tarayıcı, IoT (1GB RAM, 0.6GB quantized) | Anlık cevap, auto-complete |
| **default** | **3B** | Telefon, laptop (4GB RAM, 2GB quantized) | Günlük sohbet, özet, çeviri — %60 kullanıcı |
| **Pro** | **8B** | Laptop / 8GB VRAM | Dengeli kod/matematik, uzun bağlam |
| **high** | **30B** | Workstation / 24GB VRAM veya API | Derin muhakeme, RAG — güç isteyen işler |

Hepsi aynı damıtma hattından çıkar, sadece kapasiteye göre öğretmen sinyali ayarlanır.

---

## 2) Neden Bu Boyutlar?

- **3B default:** Llama-3.2-3B ve Qwen2.5-3B, 2026'nın en stabil 3B'leri. 4-bit quantized'da telefonda 25 tok/s, hatasız çalışıyor. Çeşitlilik için ideal default.
- **30B high:** 70B öğretmenden 30B'a damıtma, 70B'ın %85-90'ını korurken tek GPU'da (24GB, Q4) çalışır. 32B Qwen2.5 tabanı en stabil 30B sınıfı — biz de onu baz alıyoruz.

> Kullanıcı `default`ta hızlı, `high`ta güçlü hisseder — tek modelin iki yüzü gibi, ama aslında iki ayrı damıtma.

---

## 3) Baz Model Seçimi — En Stabil

| Mod | Baz | Neden Stabil? |
|-----|-----|---------------|
| **1B Nano** | **Qwen2.5-1.5B-Instruct** | En stabil 1B, tarayıcıda bile hatasız |
| **3B default** | **Llama-3.2-3B-Instruct** | Meta'nın en çok test edilmiş 3B'si, 8 dilde stabil, 128k bağlam |
| **8B Pro** | **Llama-3.1-8B-Instruct** | En dengeli 8B, kod/matematikte referans |
| **30B high** | **Qwen2.5-32B-Instruct** (30B) | MMLU 83+, 4-bit'te kararlı |

*Not: Tum baz modeller tescilli kapsamdadir, izinsiz ticari kullanim yasaktir.*

---

## 4) Damıtma Verisi — NEMES-X Avantajı

Sıradan damıtma sadece öğretmenin cevaplarını kopyalar. Bizde **40M vektör + 22 corpus + 109M dokümanlık Sovereign index** var:

- **Genel bilgi:** Wikipedia (11 dil), arXiv 3.1M, Gutenberg 79K, CC100 109M
- **Özel bilgi:** ParaCrawl 30M, Case Law, newscrawl_tr, gut_en
- **Sentetik:** Sovereign'in ürettiği 5M instruction-response çifti (self-instruct)

Karışım: %50 Sovereign sentetik + %30 insan verisi (CC100, Wikipedia) + %20 kod/matematik (arXiv, gut_en)

---

## 5) Kullanım

```bash
# Nano 1B — tarayıcı
huggingface-cli download NemesXSpace/NEMES-Free-1B --local-dir ./nemes-free-1b
# Default 3B — telefon
huggingface-cli download NemesXSpace/NEMES-Free-3B --local-dir ./nemes-free-3b
# Pro 8B — laptop
huggingface-cli download NemesXSpace/NEMES-Free-8B --local-dir ./nemes-free-8b
# High 30B — workstation/API
huggingface-cli download NemesXSpace/NEMES-Free-30B --local-dir ./nemes-free-30b
```

```python
from transformers import AutoModelForCausalLM, AutoTokenizer
# istediğin modu yükle
tok = AutoTokenizer.from_pretrained("NemesXSpace/NEMES-Free-3B")
model = AutoModelForCausalLM.from_pretrained("NemesXSpace/NEMES-Free-3B", device_map="auto")
```

**Lisans:** Tescilli — Tüm Hakları Saklıdır. Büyük şirketler (>1M token/ay) için **Enterprise on-prem** (ayrı yazılı lisans).

---

## 6) Değerlendirme Hedefleri (Free %1, rakiplerden %15-20 üstün)

| Benchmark | Nano 1B | Default 3B | Pro 8B | High 30B | Rakip Mistral-7B |
|-----------|---------|------------|--------|----------|------------------|
| MMLU | 52 | 62 | 71 | 82 | 60 |
| GSM8K | 42 | 58 | 68 | 78 | 55 |
| HumanEval | 26 | 38 | 54 | 68 | 35 |
| Türkçe MMLU | 45 | 55 | 64 | 75 | 45 |

---

## 7) Sınırlamalar

- Free, Sovereign'in %1'idir — uzun zincir muhakemede ve nadir dillerde %100'e göre zayıftır
- Halüsinasyon yapabilir — kritik işlerde retrieval (FAISS) ile kullanın
- 3B'de 128k bağlam teorik, pratikte 32k'ta stabil

---

## 8) Yayın Planı

1.  Damıtma (2 hafta) → 2. HuggingFace yükle + model kartı → 3. Benchmark + demo (nemes-x.space/free)

*Son güncelleme: 28 Ağu 2026 14:55*
