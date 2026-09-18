# NEMES-X Madenci Rehberi — Bilgi Madenciliği

> **Misyonumuz:** Dünyanın uykudaki işlem gücünü uyandırmak — onlar güç inşa
> eder, biz var olanı birleştiririz.
> **"Mine knowledge. Not hashes."**
> Sürüm 1.0 — 09 Eyl 2026. Hedef kitle: ev kullanıcısından DGX Spark sahibine herkes.

---

## 0. Neden Antminer Değil NEMES? (30 saniyelik cevap)

| | Antminer S19 Pro (2026) | NEMES madencisi |
|---|---|---|
| Ne üretir | Hash (çöpe gider) | Doğrulanmış bilgi vektörü (kalıcı index) |
| Güç | **3.250W** (elektrik sobası) | **60-240W** (ev PC'si) |
| Ses | 75 dB (oturulmaz) | Sessiz (fan sesi) |
| Günlük kâr (Ağu 2026, $0.10/kWh) | **-$3.47/gün (ZARAR)** | Pay birikimi + erken madenci bonusu |
| Donanım fiyatı (sıfır ayarı) | ~$2.500-5.500 (S19/S21) | Zaten sahip olduğun PC |
| Evde çalışır mı | Hayır (240V + 75dB + ısı) | Evet |

Antminer 2026'da evde **zarar** ediyor (kaynak: minerstat/asicminervalue, Ağu 2026).
NEMES'te aynı prizden **bilgi** çıkıyor ve pay birikiyor. Elektrik bilgiye dönüşüyor,
ısıya değil.

---

## 1. Donanım Katmanları — Sen Hangisisin?

### Katman 1: Normal PC (CPU, GPU yok)
- **Örnek:** i5/Ryzen 5, 8-16GB RAM, dahili grafik
- **Görev:** metin parse + hafif embed (CPU), denetim oyları
- **Güç:** ~65-100W sistem
- **Model:** yok (işçi modunda çalışır, model indirmez)
- **Kazanç katsayısı:** K=0.5 (destek katmanı)

### Katman 2: Oyuncu GPU'su (8-12GB VRAM)
- **Örnek:** RTX 3060 12GB, 4060/4060 Ti, RX 6700 XT
- **Görev:** embed (asıl iş) + 7B sohbet düğümü
- **Güç:** ~170-200W sistem
- **Model (7B sınıfı, HF):**
  - `Qwen/Qwen2.5-7B-Instruct-GGUF` (Q4_K_M ~4.7GB)
  - `bartowski/Meta-Llama-3.1-8B-Instruct-GGUF` (Q4_K_M ~4.9GB)
  - `bartowski/gemma-2-9b-it-GGUF` (Q4_K_M ~5.4GB)
- **Kazanç katsayısı:** K=1.0 (standart)

### Katman 3: Workstation (24GB VRAM)
- **Örnek:** RTX 4090, RTX 5090, 2x 3090
- **Görev:** embed + 30B RAG düğümü
- **Güç:** ~350-450W sistem
- **Model (30B sınıfı, HF):**
  - `Qwen/Qwen2.5-32B-Instruct-GGUF` (Q4_K_M ~19GB)
  - `bartowski/Meta-Llama-3.1-70B-Instruct-GGUF` (Q2_K ~28GB, sıkışık ama çalışır)
- **Kazanç katsayısı:** K=2.0

### Katman 4: DGX Spark (128GB birleşik bellek, 140W TDP / 240W maks)
- **Fiyat:** ~$3.999-4.699 (sıfır Antminer S21'den ucuz)
- **Görev:** 70B fine-tune + 200B'e kadar inference + embed (amiral gemisi)
- **Güç:** **140-240W** (Antminer'in **1/13'ü**)
- **Model (70B sınıfı, HF):**
  - `meta-llama/Meta-Llama-3.1-70B-Instruct` (resmi, lisanslı)
  - `Qwen/Qwen2.5-72B-Instruct-GGUF` (Q4_K_M ~41GB)
  - İki Spark ConnectX ile bağlıysa 405B'e kadar (Llama 3.1 405B)
- **Kazanç katsayısı:** K=5.0 (ağır işçi + öğretmen adayı)
- **Not:** NVIDIA verisi — 200B parametreye kadar inference, 70B'e kadar fine-tune
  tek cihazda. Bizim 70B Sovereign hattının doğal evi.

### Katman 5: Sunucu (multi-GPU / H100 sınıfı — profesyonel)
- **Örnek:** 8xH100/H200 düğümü, 2x DGX Spark bağlı (405B'e kadar), kiralık GPU sunucusu
- **Görev:** amiral modelleri çalıştırma + öğretmen düğüm (distilasyon kaynağı) + embed
- **Güç:** 700W-10kW (kurulumuna göre; veri merkezi önerilir)
- **Model (dev sınıfı, HF — hepsi açık ağırlık):**
  - `deepseek-ai/DeepSeek-V3` — 671B MoE (token başına 37B aktif), FP8 native.
    Kurulum: `sglang` veya `vLLM` (HF model kartındaki docker komutuyla).
    Gereksinim: 8x80GB sınıfı (FP8 ~670GB). Lisans: Model License (ticari kullanıma açık).
  - `moonshotai/Kimi-K2-Instruct` — 1T MoE (token başına 32B aktif), agentic/kodlama
    şampiyonu (SWE-bench Verified %65.8 tek deneme). Kurulum: `vllm serve
    moonshotai/Kimi-K2-Instruct`. Gereksinim: H100/H200 veya Blackwell (B100/B200).
    Lisans: Modified MIT.
- **Pratik alt seçenek (tek GPU'da dev tadı):**
  - `deepseek-ai/DeepSeek-R1-Distill-Qwen-32B` (Katman 3'te çalışır, ~20GB)
  - `deepseek-ai/DeepSeek-R1-Distill-Llama-70B` (Katman 4 Spark'ta çalışır)
  - Distileler gerçek V3 değil ama akıl yürütme kalıplarını taşır.
- **Kazanç katsayısı:** K=10 (öğretmen düğüm — damıtma kaynağı + en ağır iş)
- **Dürüst not:** V3/K2 tam boy tek Spark'a **sığmaz** (FP8 ~670GB/1TB).
  "Spark'ta V3 çalıştırıyorum" diyen ya distile ya quant ya API kullanıyordur.
  Katman 5 = gerçek donanım gerektirir, karşılığında en yüksek pay.

> **Kural:** Model HF'den **sen indirirsin**, biz asla kapalı binary dayatmayız.
> Miner açık kaynak, pipeline tescilli. Lisanslar: model kartındaki lisansa uyarız
> (Llama Community, Qwen, Gemma şartları kullanıcıya aittir).

---

## 2. Kazanç Mantığı (coin + halving, sadeleştirilmiş)

1. **Pay:** `S = V` (V = doğrulanmış vektör; herkes aynı oranda — katman
   katsayısı henüz yok, bkz. B27 merit-tier tasarımı).
   Vektörün %10'u rastgele yeniden hesaplatılır (spot-check); tutmazsa pay yok.
2. **Batch:** 20 kanıt = 1 batch. Batch ödülü taban **0.002 NEMES**.
3. **Halving:** her 5M batch'te ödül yarıya iner (erken madenci bonusu).
4. **Havuz:** `H = gelir × %50` (Enterprise + ücretli API geliri).
5. **Ödeme:** `pay × (H / toplam_pay)`. Minimum yok, toz birikir.
6. **Faz 1 (bugün):** pay deftere işlenir, birikir. Nakit payout Faz 2'de
   (Hazine + Enterprise geliriyle).

**Dürüst not:** Coin'in borsa değeri bugün yok — biriktirdiğin pay, ağın
büyümesiyle değerlenir. Antminer'ın bugünkü **garanti zararına** karşı bizim
vaadimiz: düşük elektrik + biriken pay + erken madenci çarpanı. Hesabını buna
göre yap.

**Elektrik hesabı (örnek, $0.15/kWh Türkiye):**
- Antminer S19 Pro: 3.25kW × 24s × $0.15 = **$11.7/gün gider**, gelir ~$4 → **-$7.7/gün**
- Oyuncu PC (0.2kW): **$0.72/gün gider** + pay birikimi
- DGX Spark (0.2kW): **$0.72/gün gider** + 5x katsayıyla pay birikimi

---

## 3. Kurulum (5 dakika, 3 adım)

### Adım 1 — İndir
- `nemes-x.space` → Download → işletim sistemini seç (Windows NSIS / Ubuntu .deb)
- İmza: NSIS imzalı binary, SHA256 sitede yayınlanır. Tutmayan binary'yi çalıştırma.

### Adım 2 — Anahtar + (varsa) depolama
```sh
nemes-miner keygen
nemes-miner keygen --storage --yol ~/.nemes/depolama --boyut-mb 1024
```
- `--storage`: 100GB baraj + disk sınavı (1GB yaz+oku+doğrula+sil). Geçemezsen
  compute-only madenci olursun (yine kazanırsın, K aynı, depolama ücreti yok).

### Adım 3 — Çalıştır
```sh
# Katman 2 örneği (oyuncu GPU):
nemes-miner mine --simple --komuta https://komuta.nemes-x.space \
  --embed-api http://127.0.0.1:1241 --corpus tr --depolama ~/.nemes/depolama

# DGX Spark (ev ağı, P2P açık):
nemes-miner mine --simple --komuta https://komuta.nemes-x.space \
  --embed-api http://127.0.0.1:1241 --corpus tr \
  --depolama ~/depolama --p2p-port 4003 --p2p-dinle
```
- `--p2p-dinle`: görev duyurusu (`nemes/gorev`) gelince beklemeden uyanır.
- İlk shard ilanı otomatik yayınlanır, 30sn'de bir yoklanır (re-claim).
- Çıkış: `Ctrl+C` (graceful durur, yarım iş komutada kalır, kayıp yok).

### Adım 4 — (Opsiyonel) Model indir — sohbet düğümü olmak için
```sh
# 7B örneği (Katman 2):
huggingface-cli download Qwen/Qwen2.5-7B-Instruct-GGUF \
  --include "*Q4_K_M*" --local-dir ~/modeller/qwen7b
# Miner ayarlarından "sohbet düğümü" aç, model yolunu göster.
```
- 70B (Katman 4): `huggingface-cli download meta-llama/Meta-Llama-3.1-70B-Instruct`
  (HF erişim onayı gerekir — meta'nın sayfasından iste).
- Model lisansı sana aittir; ticari kullanımda model kartını oku.

---

## 4. Paralel Ağ ile Entegrasyon (neden tek başına değilsin)

- **Görev:** komuta dağıtır (HTTP) + mesh'e ilan eder (`nemes/gorev` GossipSub).
- **Doğrulama:** her kanıtın %1'i başka madenciye denetletilir (eş-doğrulama).
- **Depolama:** parçaların 3 kopyası mesh'te durur; düşen disk otomatik onarılır
  (C7), sahtekar yoklamayla atılır (C8). Senin diskin de ağın parçası olur.
- **Kesinti:** fişi çek, geri gel — kaldığın yerden devam (cursor + resume).
  Ceza yok, slashing yok (3 strike sadece hileye).

---

## 5. SSS

**Bilgisayarım ısınır mı?** Embed GPU'yu %30-100 kullanır. Miner `--gpu` ayarı +
sistem fan eğrisiyle 70-75°C bandında tutulur. Antminer gibi 75dB yok.

**İnternet kotası yer mi?** Görev metinleri KB'lar mertebesi. Modeli bir kez
indirirsin (5-40GB), sonrası küçük trafik. Depolama katmanında parça transferi
olur (64MB parçalar), ayardan sınırlanabilir.

**Coin ne zaman para eder?** Borsa + likidite gelince (Faz 2). Bugün biriktir,
erken madenci çarpanıyla. Garanti yok — ev elektriğinle Antminer zararından
kaçmak bugünkü kazancın.

**Vergi?** Ülkenizin kripto gelir kuralı geçerli. Muhasebe için `bakiye` ve
`ledger` endpoint'lerinden döküm alabilirsin. Mali müşavire danış.

**Kapatmak istersem?** `Ctrl+C`. Payın defterde kalır, geri gelince devam.
Depolama taahhüdün varsa 3 nabız kaçırmadan önce haber ver (ölüm ilanı yeme).

---

*Son güncelleme: 09 Eyl 2026. Rakamlar: NVIDIA DGX Spark datasheet (128GB/240W),
BT-MINERS/Kryptex/minerstat (S19 Pro 3250W, Ağu 2026 negatif kârlılık). Fiyatlar
değişir — yatırım kararı öncesi güncel bak.*
