# NEMES-X GÖREV DURUMU — 31 Ağu 19:50
> ANAYASA v3.0 (yerel /srv/beyin, 320 madde). GITHUB'DA YOK.

## TAMAMLANAN ✅ — GÜNCEL KRİTİK YOL
- **#1 ULTRA 4K60 render bitir + encode + publish — DONE ✅** (31 Ağu 19:50 işaretlendi, %97 → %100)

## TAMAMLANAN ✅

### 1. Site v3 — Kurumsal Seviye
- **Tema Sistemi**: Beyaz/Siyah + Altın sabit, Sistem tercihi (sistem tercihine göre), FOUC'suz
- **8 Dil**: EN TR DE FR ES RU AR ZH — Sağ üst dil menüsü, localStorage, AR RTL otomatik
- **Manifesto**: 103 anahtar, 13K kelime × 7 dil = %100 kapsama (EN fallback)
- **Video**: 180s (4K 41M + 1080p 24M), müzik loop düzeltildi (aloop), encode bitti
- **Tarife**: `$5/$15/$40` kaldırıldı → **NEMES-FREE** (Tescilli, tüm hakları saklıdır, %1) / **NEMES-ENTERPRISE** (100% Sovereign, şirketler için ücretli, H=gelir×%50)

### 2. Çeviriler (8 Dil Tamam)
- EN TR DE FR ES RU AR ZH — 10 sayfa × 8 dil = %100 anahtar kapsamı
- Manifesto: 103 anahtar/dil (13K kelime) — TR/DE/FR/ES/RU/AR/ZH tam çevirildi
- Rakamlar/TR/EN/DE/FR/ES/RU/AR/ZH çevrildi (nav_rakamlar, rak.title, Milyar birimleri)
* Manifesto CTA kutusu + Rakamlar Milyar hücreleri i18n'e alındı

### 3. Site Altyapısı
- **Tema**: Beyaz/Siyah + Altın sabit, Sistem tercihi (prefers-color-scheme), FOUC'suz
- **i18n**: data-i18n + shared.* + sayfa özel anahtarlar, localStorage + prefers-color-scheme
- **Tüm sayfalar**: hreflang (8 dil + x-default), OG/Twitter meta, canonical, sitemap.xml (10 URL)
- **404 sayfası**: Nav + Footer + dil/tema kontrolleri + i18n
- **Sitemap**: 10 URL (hreflang + changefreq/priority)
- **Eski videolar**: Repo'dan kaldırıldı (3 dosya silindi)
- **Eski tarifeler**: `$5/$15/$40` kaldırıldı → Free / Enterprise 2-kademe model

## DEVAM EDEN (Arka Plan) — CANLI (01 Eyl 12:15)
- **#1 ULTRA 4K60**: DONE ✅ (31 Ağu 19:50)
- **#2 newscrawl_tr**: 28G emb (12 shard) — dün 11G’ydi, +17G/15saat — 12 işçi aktif — ETA ~20 saat (02 Eyl sabah) — beyin_sirasi.py zincirinde
- **#3 gut_en**: %85.9 (4.57M/5.33M) — PAUSED, newscrawl_tr bitince resume
- **Disk**: 139G boş / 916G (%85) — 01 Eyl 11:50 — arşiv sonrası (enwiki 25G + trwiki 1G + buyuk_diller 41G → /beyin_arsiv) — wikidata 169G korundu
- **Isı**: Tctl 61.4°C / NVMe 45.9°C — hedef 70-75 içinde, stabil
- **Embed API**: 6× llama-server (1241-1246) — 107% (1242), diğerleri idle/0.2%
- **Miner v0.2.0**: Tauri iskelet + Komuta API hazır (NemesXSpace/miner/, /srv/beyin/kaynaklar/komuta_api.py) — imzalı NSIS, HF 4 model, Ollama gibi pull

## KALAN YAPILACAKLAR (P0/P1)
1. **Manifesto çevirileri** (7 dil × ~13K kelime) — şu an EN fallback, 7 dil bekliyor
2. **OГ/Twitter meta + hreflang** — 8 dil GitHub Pages'e deploy bekliyor (git push ağ sorunu)
3. **Eski video dosyaları** — Repo'da hala `nemes-x-promo.mp4`, `nemes-x-promo-en.mp4`, `nemes-x-promo-tr.mp4`, `nemes-x-promo-music.mp4` (temizlenebilir)
4. **404 sayfası** — Nav/Footer i18n eklendi, push bekliyor
5. **Eski video dosyaları** — Repo'da hala `nemes-x-promo.mp4`, `nemes-x-promo-en.mp4`, `nemes-x-promo-tr.mp4`, `nemes-x-promo-music.mp4` (temizlenebilir)
6. **Manifesto çevirileri** — 7 dil × ~13K kelime (~150K token) — P0 ama zaman alıcı
7. **Kalan diller** (DE FR ES RU AR ZH) manifesto + sayfa çevirileri

## BLOKER
- **Git push ağ sorunu** — GitHub'a push zaman aşımına uğruyor (ağ sorunu olabilir)
- **Manifesto çevirileri** — 7 dil × 13K kelime (en büyük kalan iş)

## SİSTEM DURUMU — 01 Eyl 12:15 CANLI
- Disk: 139G boş / 916G (%85) — arşiv sonrası +58G net — / 383G boş — `beyin-sunucu.service` enabled, 6 llama sunucusu auto-start
- Beyin: newscrawl_tr 28G emb (dün 11G), gut_en %85.9, wikidata 169G korundu — Tctl 61.4°C ideal
- RAM: 30G total, 8.3G used, 21G available — yeterli (damıtma madencide, merkezde değil)
- Bekçi: beyin_bekci 6 port, son restart 19:40, nemes-izle 60sn logluyor (NemesXSpace/izle.log)
- Komuta API: /srv/beyin/kaynaklar/komuta_api.py (8787) — iskelet hazır, uvicorn ile test edilecek
- Miner: NemesXSpace/miner/ — Tauri + HF 4 model (Qwen3B/Gemma2B/Qwen7B/Qwen14B) — imzalı NSIS planlandı
- GitHub push ağ sorunu: timeout (beklemede)

## KRİTİK YOL — REVIZE (Merkez = Bilgi Merkezi, Madenciler Üretir)
```
1. ULTRA DONE ✅ (31 Ağu 19:50)
   ↓
2. newscrawl_tr 28G → ~50G — ETA 02 Eyl sabah (12 işçi) — ŞU AN ÇALIŞIYOR
   ↓
3. gut_en resume → wet_en → paracrawl/caselaw — Beyin 109M → 40M vektör
   ↓
4. FAISS PQ index (Madde 18) — madencinin RAG’i
   ↓
5. Miner v0.2 Windows (Tauri, imzalı, HF pull, Ollama gibi) — iskelet hazır
   ↓
6. Komuta API + Ödül Faz1/2 — pay→API, Enterprise H×%50
   ↓
7. Debian/Ubuntu port — şirketler için
```
Damıtma merkezde değil, madencide — hazır 32B öğretmen, 1B/3B öğrenci QLoRA ile madenci GPU’sunda.

**Sonraki adım**: `komuta_api`’yi uvicorn ile dry-run + miner `npm install` (Rust CI’da build) → website download güncelle