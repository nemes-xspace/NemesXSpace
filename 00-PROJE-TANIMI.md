# NEMES-X — Proje Tanımı (Baştan Sona)
**Sürüm:** 1.0 — 28 Ağustos 2026
**Durum:** Canlı (nemes-x.space) — BEYİN nicelemesi + ULTRA video paralel
**Sahibi:** d3str0y1ng
**Kayıt yeri:** `~/NemesXSpace/00-PROJE-TANIMI.md` (bu dosya) — tek hakikat kaynağı

---

## 0) Özet — Bir Cümlede NEMES-X

**NEMES-X, milyonlarca cihazın boştaki GPU'sunu hash değil bilgi madenciliğine yönlendirip, ortaya çıkan en güçlü yapay zekanın tam kontrolünü tek elde tutarken, halka gücün %1'lik damıtılmış ve ücretsiz bir versiyonunu veren merkeziyetsiz-egemen bir bilgi ağıdır.**

Slogan: **"Mine knowledge. Not hashes."**

**Misyon:** Dünyanın uykudaki işlem gücünü uyandırmak — onlar güç inşa eder, biz var olanı birleştiririz. Büyük firmalar her FLOP'u satın almak zorundadır (arazi, trafo, izin, yıllar); biz evlerde zaten takılı milyonlarca boş GPU'yu ödünç alırız. Engel para değil, fiziktir — ve fizik bizden yanadır.

---

## 1) Problem

1.  **Bilgi birkaç şirketin elinde** — Wikipedia, arXiv, Common Crawl gibi açık kaynaklar bile tek merkezlerde işleniyor.
2.  **Hash mining faydasız** — Bitcoin için harcanan elektrik, şehirlerin elektriğini kesecek kadar büyük ama hiçbir kalıcı bilgi üretmiyor.
3.  **En güçlü AI kapalı ve tekelleşmiş** — Açık modeller zayıf, kapalı modeller denetlenemez ve pahalı.

## 2) Çözüm — NEMES-X

Boştaki GPU'lara **hash bulmacası değil, embedding işi** yaptırmak:

```
AÇIK VERİ (109M doküman, 22+ corpus)
    ↓ parse (temizle → bağlam-duyarlı pasajlara böl)
GPU EMBEDDING (nomic-embed-text-v1.5, int8 quantized)
    ↓ %10 rastgele yeniden hesapla, cosine ≥0.99 doğrula
KALICI BİLGİ INDEX'İ (FAISS PQ, sansüre dayanıklı, aranabilir)
    ↓ RAG → Sovereign Brain
```

Madenci GPU'sunu çalıştırır, doğrulanmış her vektör için **pay (share)** kazanır: `S = V × K` (V=vektör, K=korpus katsayısı). Haftalık havuz `H = gelir × %50`, pay değeri `P = H / toplam_pay`, ödeme `pay × P`. Minimum yok, toz birikir.

---

## 3) Mimari — 4 Ana Bileşen

### A) BEYİN — Kalıcı Yapay Hafıza (`/srv/beyin`)
- **Anayasa:** `BEYIN/ANAYASA.md` (12 madde — tek komuta, sıra değişmezliği, done damgası, doğrulama zorunluluğu, silme yasağı, tek nefes kuralı...)
- **Veri tabanı:** `beyin.db` (SQLite + FTS5) — deneyim/hata/bilgi tabloları, `beyin.py` ile yönetilir
- **Pipeline:** `beyin_sirasi.py` zinciri — 22+ korpus sırayla:
  `trws/trwq/trwt → enwt/enws/enwq → de/fr/es/ru/it/pt/ar/ja/zh → arxiv → tat_tr/tat_en → cc100_tr (109M, 73GB, BİTTİ) → newscrawl_tr → gut_en (5.3M, 44GB, DEVAM) → wet_en → paracrawl → caselaw`
- **Niceleme:** `wiki_embed_par.py` (12→2 işçi, MAX_CHAR 4500) + 6× `llama-server` (nomic Q4_K_M, 1241-1246) + `embed_bekci.sh` (45sn'de bir donuk kontrol, 10dk cooldown ile restart)
- **Durum (28 Ağu 13:45):** cc100_tr `tamam` (73.1GB, 12 shard), gut_en 2 işçiyle embed'de (~37/s ramp-up), disk `/srv/beyin` 188GB boş, ısı 70-75°C hedef

### B) Miner — `nemes-x-miner` (`/srv/beyin/website/miner`)
- `No root`, sandboxed worker, imzali binary, kapali tescilli pipeline, tum haklari saklidir
- Kullanıcı `nemes-x.space`'ten indirir, normal kullanıcı olarak çalıştırır

### C) Video — Promo Film (`/srv/beyin/website/video`)
- **Mevcut canlı:** `nemes-x-promo-en-135s.mp4` — 1080p30, 21.3MB, 135sn, EN anlatım + ducking müzik, gerçek logo (eğri şeritler, Nx, NETWORK)
- **ULTRA (devam):** `generate_frames_ultra.py` — 8K→4K SSAA, 12 örneklem motion blur, subpixel Ken Burns, soft-knee bloom, filmik grade, 8100 frame @60fps, 4K x265 + 1080p60, 7881/8100 (%97), 4 worker nice19, 70-75°C bekçili

### D) Website — `nemes-x.space` (`/srv/beyin/website`)
- `index.html` (video embed, download, miner), `verify.html`, `press.html`, GitHub Pages (`nemes-xspace.github.io`, token ile publish)

---

## 4) Model Stratejisi — Matruşka Freemium (KARAR)

**Vizyon:** En guclu modelin %100'u tek elde, tum haklari saklidir. Hicbir surum ucretsiz/acik degildir.

| Katman | Güç | Kitle | Lisans | Teknik |
|--------|-----|-------|--------|--------|
| **NEMES-Free** | %1 (7B sinifi, 40M vektorden damitildigi icin rakiplerinden %15-20 guclu) | Herkes, bireyler | **Tescilli — Tum Haklari Saklidir, yazili izinle** | 70B ana beyinden damitma + FAISS retrieval |
| **Modlar** | `default: 3B` (hızlı/telefon) / `high: 30B` (güçlü/API) | Çeşitlilik için | Aynı Free içinde mod seçimi | Tek damıtma, çok mod |
| **NEMES-Enterprise** | **%100** (70B+ Sovereign) | Şirketler (>1M token/ay veya 20+ çalışan) | **Ücretli** — Cloud API veya **On-Premise** (kendi DC'sinde, yıllık lisans) | Büyük şirket halka açık hattı taciz etmesin |

**Finansman (2 aşamalı, 28 Ağu kararı — `docs/TOKENOMI.md`):** Faz 1'de Madenci=Müşteri (API hakkı, hazineye dokunma), Faz 2'de Hazine + Enterprise × %50 → nakit payout

---

## 5) Güvenlik ve Güven

- **Doğrulama:** Her vektörün %10'u rastgele yeniden hesaplatılır, cosine <0.99 ise pay yok
- **Gizlilik:** Pipeline tescilli ve kapalidir, izinsiz kullanim yasaktir (ANAYASA Madde 4)
- **Kapalılık:** Son birleşmiş index ve 70B ağırlıklar kapalı kalır — güç sende, güven açık pipeline ile sağlanır

---

## 6) Yol Haritası

- [x] CC100 parse + embed (73GB, bitti)
- [ ] gut_en → wet_en → paracrawl → caselaw embed zinciri
- [ ] ULTRA 4K60 bitir + publish (7881→8100)
- [ ] NEMES-Free model kartı (3B vs 7B stabil seçimi) + HuggingFace yükle + benchmark
- [ ] Enterprise on-prem lisans metni + API rate limit

---

## 7) Kayıt Disiplini

- Bu dosya (`00-PROJE-TANIMI.md`) projenin ne olduğunu tanımlar — değişirse burası güncellenir, sürüm artar
- `README.md` günlük yapılan/yapılacak değişikliklerin kronolojisidir
- `beyin.db` ve `sira.log` teknik hakikat kaynağıdır

*Son güncelleme: 28 Ağu 2026 14:50 — bir sonraki değişiklikte burası ve README birlikte güncellenecek.*
