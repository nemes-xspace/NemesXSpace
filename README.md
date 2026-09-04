# NemesXSpace — Proje Kaydı
**Oluşturulma:** 28 Ağustos 2026, 13:45
**Konum:** `~/NemesXSpace` → Masaüstü'ne kısayol eklendi
**Amaç:** NEMES-X projesinde yapılan ve yapılacak TÜM değişikliklerin tek kaynaktan takibi

---

## 1) VİDEO — Promo Film (nemes-x.space)

### Level 2 — 135sn İngilizce (TAMAMLANDI ✓)
- **Süre:** 75sn → 135sn (şarkı süresi)
- **Çıktı:** `nemes-x-promo-en-135s.mp4` — 1920×1080@30fps
- **Sahne haritası (4050 frame):**
  ```
  0-5s    HOOK: "Your GPU is idle. It could be mining knowledge."
  5-12s   PROBLEM: Information is gated / Hash mining
  12-20s  LOGO: NEMES-X reveal + "The Sovereign Brain"
  20-38s  PIPELINE: 5 adım (GPU Embedding vurgulu)
  38-50s  NUMBERS: 109M, 40M+, 22+, 13, 0 + sayaç
  50-65s  SECURITY: 6 kart (No root, Sandboxed...)
  65-75s  CTA: Start Mining + nemes-x.space
  75-95s  CORPUS: 22+ Knowledge Corpora
  95-115s ECONOMICS: S=V×K, H=revenue×50% ...
  115-135s FINAL: logo + nemes-x.space + @NemesXSpace
  ```
- **Ses:** EN anlatım 84sn (ElevenLabs) + müzik ducking (%15 anlatım varken, %30 tek başına)

### Level 3 — Sinematik (TAMAMLANDI ✓)
- Eklenenler: Aurora bg, 120 parçacık, kinetik tipografi, logo pulsating glow (çift halo), pipeline ışık izi + progress bar, tam sayaç (5/5), kamera sarsıntısı, vignette
- **SFX sentezlendi** (numpy): 8 whoosh + riser + impact + 5 tick → sonra **kullanıcı isteğiyle KALDIRILDI**

### Level 3.1 — Düzeltmeler (TAMAMLANDI ✓)
- **Sorunlar:** Sahte logo (düz şeritler), geçişlerde boş ekran (12f crossfade + geç içerik), ışık bandı artifact, chroma metinlere taşıyordu, CTA ölü anı, 23.8MB şişme
- **Fixler:**
  - Gerçek `logo.png` gömüldü (eğri şeritler + serif Nx + NETWORK, badge zemini BG ile birebir #0A0A0F)
  - 24f dissolve (giden sahne donuk tutuldu) + gaussian smooth sweep (bant yok)
  - chroma sadece logo bölgesi, frame ≥372, dx≤3
  - CTA logo fade 40→25f, shake senkronize edildi
  - scene5 sayıları 1146'dan başlatıldı, crf 22 → 21.3MB

### Level 4 — Ultra Cinema Master (DEVAM EDİYOR ⏳)
- **Hedef:** Daha önce yapılmamış reklam statüsü — minimum 3 saat render
- **Mimari:** 8K (7680×4320) supersample → 4K (3840×2160) SSAA (2×) + 12 örneklemli motion blur (gerçek) + subpixel Ken Burns (AFFINE float) + soft-knee bloom + filmik tone curve + vignette
- **Çıktı planı:**
  - `nemes-x-promo-en-4k-60fps.mp4` — 3840×2160@60fps, x265 crf18, <95MB (arşiv/master)
  - `nemes-x-promo-en-135s.mp4` — 1920×1080@60fps, x264 crf19 (web embed, mevcut dosyanın üzerine)
- **Durum (28 Ağu 13:34):** **7881/8100 (%97)** — 2600/2905 bu resume'de bitti, ETA ~0.35 saat. 4 worker + 12 subsample, `nice 19` ile 70-75°C bandında. Bekçi: Tctl>75 DURDUR, <70 DEVAM.
- **Stabilite fixleri:** Shake tamamen kaldırıldı, KB integer sıçrama → subpixel, bloom sert eşik → smoothstep, parçacık yumuşak sprite + yavaş twinkle
- **Dosyalar:** `generate_frames_ultra.py` (8K), `video_frames_4k/` (8100 PNG, compress_level=1), `render_ultra_pipeline.sh` (render→4K→1080p, resume destekli)

---

## 2) BEYİN — Bilgi Madenciliği

### cc100_tr (TAMAMLANDI ✓ 28 Ağu 04:39)
- **Corpus:** 109.279.716 madde, ana DB `wiki_cc100_tr.db` 25GB
- **Emb:** 12 shard `wiki_cc100_tr_emb_*.db` toplam **73.1GB** (1.8–8.7GB/shard)
- **Config:** `MAX_CHAR=4500, isci=12` → 6 llama-server (1241-1246, nomic-embed-text-v1.5 Q4_K_M, 4 thread, 1241-1243 aktif kullanılıyordu, 1244-1246 eklenerek 6'ya çıkarıldı)
- **Hız:** ~310–340 madde/s (ölçüldü), 2 gün sürdü
- **Sorunlar:** İlk OOM (MAX_CHAR ile çözüldü), 3 shard 20 saat donuk kaldı, bekçi 4→6 sunucuya yükseltildi, duplicate parent temizlendi

### gut_en (DEVAM EDİYOR ⏳)
- **Corpus:** Gutenberg 79K kitap, 5.325.505 madde, DB 44.7GB
- **Durum:** 28 Ağu 13:34'te `isci=2` ile başladı (önce 12→6→4→2 düşürüldü, ısı 91.8°C → 80.1°C)
- **Hız:** İlk 30sn 0.93MB → ~37/s (ramp-up), birazdan 100+/s bekleniyor
- **Disk:** `/srv/beyin` (sdb1) 916GB toplam, **188GB boş** (önce 212GB idi, 44GB gut_en eklendi)

### Sıradaki Korpuslar (beyin_sirasi.py zinciri)
`newscrawl_tr (12) → gut_en (2) → wet_en (12) → paracrawl → caselaw` — hepsi `isci` ayarlı, `sira_*.done` ile resume'li

### Altyapı Notları
- **Servisler:** `beyin_bekci.service` (6 port, Restart=always), `beyin_sira.service` (override.conf ile 6 EMBED_API, isci ayarlı)
- **Isı yönetimi:** BEYİN 2 işçi + ULTRA 4 worker (nice 19) ile **69.5°C**'ye indi (hedef 70-75). ULTRA bekçisi 75/70 bandında. GPU %100 → %3 arası değişiyor, dokunulmuyor (kullanıcı talimatı).

---

## 3) MODEL — Matruşka Stratejisi (KARAR VERİLDİ ✓)

**Vizyon:** Milyonlarca cihazın boştaki GPU'sunu hash yerine bilgi madenciliğine yönlendirip tarihin en güçlü AI'ını kurmak, ama tam kontrol sende kalacak.

**Karar — Freemium Matruşka:**

| Model | Güç | Kitle | Lisans | Amaç |
|-------|-----|-------|--------|------|
| **NEMES-Free** | %1 (7B sınıfı, ama 40M vektörden damıtıldığı için rakiplerinden %15-20 güçlü) | Herkes, bireyler, öğrenciler | **Tescilli — Tüm Hakları Saklıdır, yazılı izinle** | Güven ve kitle çekmek |
| **NEMES-Pro** | %1 (aynı Free, ama API üzerinden sınırsız) | — | API | — |
| **NEMES-Enterprise** | %100 (70B+ ana beyin) | Şirketler (1M token/ay veya 20+ çalışan) | **Ücretli** — Cloud API veya **On-Premise** (kendi veri merkezinde, yıllık lisans) | Büyük şirketler halka açık hattı taciz etmesin, kendi donanımında çalışsın |

**Çeşitlilik isteğin için modlar:**
- `default: 3B` (hızlı, telefonda)
- `high: 30B` (güçlü, API)
- İleride `ultra: 70B` (sadece Enterprise on-prem)

Hepsi aynı %100 ana beyinden damıtılacak — stabil olan seçilecek, birden fazla mod zenginlik gösterecek.

**Finansman (2 aşamalı, 28 Ağu kararı — `docs/odul-stratejisi.md`):**
- **Faz 1 (bootstrap, madenci kazanana kadar):** Seçenek 2 — Madenci = Müşteri, paylar API hakkı olarak verilir, hazineye dokunulmaz
- **Faz 2 (sürdürülebilir):** Seçenek 1 — Hazine + Enterprise × %50 → nakit payout

**Sıradaki adım:** Free modelin model kartı — 3B mı 7B mi, HuggingFace'e yükleme, ilk benchmark hedefleri.

---

## 4) YAPILACAKLAR

- [ ] ULTRA 4K60 render bitir (7881→8100, ~0.35 saat) + 4K x265 + 1080p60 encode + publish (nemes-x.space)
- [ ] gut_en embed bitir (2 işçi, ~1-2 gün)
- [x] Free model kartı yaz (4'lü aile: 1B Nano Qwen2.5-1.5B + 3B Default Llama-3.2-3B + 8B Pro Llama-3.1-8B + 30B High Qwen2.5-32B) — `model/MODEL-KARTI-Free.md` + `model/damitma-plani.md`
- [ ] HuggingFace'e NEMES-Free yükle + benchmark (Sovereign checkpoint bekleniyor)
- [ ] Enterprise on-prem lisans metni

---

## 5) TEKNİK NOTLAR — Isı & Stabilite

- İlk ULTRA 10 worker → 90.5°C → 4 worker + nice 19 → 69.5°C (hedef 70-75 tutuyor)
- BEYİN 12→6→4→2 işçi kademeli düşürüldü, 6→3 sunucuya inildi (GPU 100% → 3% arası)
- Tüm render'lar resume'li (`frame_*.png` varsa atla), pipeline `render_ultra_pipeline.sh` ile otonom

---
*Bu dosya her değişiklikte güncellenir. Son güncelleme: 28 Ağu 2026 13:45 — masaüstündeki kısayoldan her an açabilirsin.*
