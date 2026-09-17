# NEMES-X GÖREV DURUMU — 17 Eyl 13:50 (senkron)
> ANAYASA v3.0 (yerel /srv/beyin, 320 madde). GITHUB'DA YOK.

## TAMAMLANAN ✅ — GÜNCEL KRİTİK YOL
- **#1 ULTRA 4K60 render bitir + encode + publish — DONE ✅** (31 Ağu 19:50 işaretlendi, %97 → %100)
- **#2 cc100_tr embed + merge + FAISS-24 — DONE ✅** (15 Eyl: 109.271.443 vektör, 118G merge, 4.1G index, ntotal doğrulandı; 16 Eyl arşivlendi)
- **#3 Denetim sertleştirme B-1→B-5 — DONE ✅** (16 Eyl: escrow+%10 salt, retry, hash pini, parse resume; migration 012+013 canlıda)
- **#4 Komuta OOM + denetim zehiri + toplu-kanıt — DONE ✅** (17 Eyl: artımlı havuz RSS 5G sabit, kanarya filtresi kuyruğu eritiyor -15.7K/sa, `/api/kanit/toplu` cap 100; test 12/12)
- **#5 İlk canlı mesh + shard disiplini — DONE ✅** (17 Eyl: miner-b gossip bağlı, claimler newscrawl 1.72M'de, istek 0.76/sn/miner, kilit 0ms)
- **Canlı (17 Eyl 13:50):** 2.735M kanıt, kuyruk 168K (eriyor), 136.941 batch, 259.55 NEMES, 3 miner (1.21M/1.23M/299K pay)

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

## DEVAM EDEN (Arka Plan) — CANLI (08 Eyl 00:05)
- **#1 ULTRA 4K60**: DONE ✅ (31 Ağu 19:50)
- **Tam eğitim (adapter-v01)**: DONE ✅ 5900/5900 (loss 1.4835, acc 0.693) — 22:33 bitti — final test yapıldı: 10 soruda 4 iyi / 6 zayıf-tekrarlı (1.7B + LoRA için normal, RAG gerekli) — `yanit-final.json`
- **wet_en rebuild**: BAŞARISIZ ❌ 08 Eyl 01:02 — 34.4M maddeye kadar geldi (8690/s), sonra `database or disk is full` — 111G txt → 226G DB'yi aştı, disk %100 oldu — yarım 226G DB silindi, disk 226G boşa döndü (%75) — karar bekliyor: atla / parçalı / şema düzelt
- **#2 newscrawl_tr**: TAMAM ✅ (01 Eyl 17:09, 12 shard) — `sira_newscrawl_tr.done`
- **#3 gut_en**: TAMAM ✅ (01 Eyl 23:12, 5.3M madde, 46G DB + ~3.9G emb) — `sira_gut_en.done`
- **Takılanlar**: wet_en (DB malformed) / paracrawl (S3 404) — `eksik-kaynak`; caselaw HAZIR ✅ (Illinois vol 1 ALTO→JSONL: 450 görüş → 454 madde, wiki_caselaw.db 2.5MB; cc100 bitince done silinip tetiklenecek)
- **Pipeline**: TAMAM ✅ (08 Eyl 10:50 ilk temiz tur) — yamalı kod + `beyin_sira.service` — enwt/wet_en/paracrawl/caselaw `eksik-kaynak` işaretli, diğerleri `tamam` — turlar artık anlık geçiyor, gerçek düzeltme gelince ilgili done silinip retry edilir
- **Disk**: /srv/beyin 105G boş / 916G (%89) — 07 Eyl 15:47 — / 325G boş — 01 Eyl'deki 132G'den düştü, takipte
- **Isı/Güç**: GPU 60°C, 150W limit, %100, VRAM 6.4/11G — CPU powersave 2.96GHz, Tctl ~58°C — stabil
- **Embed API**: 6× llama-server (1241-1246) + bundle :1251 — idle, restart yok
- **Testnet**: komuta-rs x2 + 2 miner canlı (bugün 01:42'den beri)
- **Miner v0.2.0**: Tauri iskelet + Komuta API hazır (NemesXSpace/miner/, /srv/beyin/kaynaklar/komuta_api.py) — imzalı NSIS, HF 4 model, Ollama gibi pull
- **P2P görev dağıtımı**: CANLI ✅ (08 Eyl ~11:00 deploy) — komuta+2 miner restart, hata yok — corpus tr tükenik (204) olduğu için ilk duyuru bir sonraki dağıtımda ateşlenecek
- **S2/S3/S4**: KAPANDI ✅ (08 Eyl) — miner-api üyelikten çıkarıldı, komuta_api.py donduruldu, 6 iskelet `_arsiv/`'de, damıtma ertelendi (Qwen3-LoRA kilit) — `cargo check` temiz

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

## SİSTEM DURUMU — 16 Eyl 11:15 CANLI (cc100 KAPANIŞ)
- cc100_tr embed: TAMAM ✅ 109.271.443 vektör (%101.2, 7.635 atlanan) — 15 Eyl 12 işçi 0'la kapattı, merge 1.3sa + FAISS-24 66dk bitti (ntotal doğrulandı)
- FAISS: 24 index ✅ (cc100: 4.1G faiss + 834M ids, nlist 1024) — önceki 23 toplam 3.4G
- Testnet: komuta+2 miner canlı (P2P kodlu binary) — corpus tr tükenik (204)
- Site: CANLI ✅ (09 Eyl) — 404 çözüldü (repo private→public + Pages açıldı), root/miner/faq 200
- İzleme: `nemes-izleme` timer+servis sağlıklı (journal'da 15dk kontroller, 0 alarm) — eski `izle.log` emekli, artık journal'a bakılır
- Pipeline: `beyin_sira` anlık TAMAM turlarında (günde ~4000 restart normal, hepsi skip) — `sira.log` 18M (çift satır: servis+script ikisi de yazıyor, düşük öncelikli temizlik)
- Disk: /srv/beyin ~215G boş (%76), / ~82G boş — emb 12x7G + orig-bak 25G `/home/.../nemes-merge/arsiv/`'de (16 Eyl taşındı, silinen yok; merge kapsaması 1800 örnekle doğrulandı, 0 hata)
- Reboot: 16 Eyl ~11:12 (uptime taze) — llama 0/6 (görev yok, bekliyor), servisler 6/6 active, yedek 00:02 taze
- Eğitim: 4750/5900, loss 1.48, ETA ~8sa — bitince 10 soruluk test + final adapter
- RAM: 30G total — yeterli (damıtma madencide, merkezde değil)
- Bekçi: beyin_bekci 6 port, nemes-izle 60sn logluyor (NemesXSpace/izle.log — son satırlar 02 Eyl'de kalmış, izle timer durmuş olabilir, kontrol edilecek)
- Komuta API: /srv/beyin/kaynaklar/komuta_api.py (8787) — iskelet hazır, uvicorn ile test edilecek (eğitim sonrası)
- Miner: NemesXSpace/miner/ — Tauri + HF 4 model (Qwen3B/Gemma2B/Qwen7B/Qwen14B) — imzalı NSIS planlandı
- GitHub push ağ sorunu: timeout (beklemede)

## KRİTİK YOL — REVİZE 07 Eyl (Merkez = Bilgi Merkezi, Madenciler Üretir)
```
1. ULTRA DONE ✅ (31 Ağu 19:50)
   ↓
2. newscrawl_tr TAMAM ✅ + gut_en TAMAM ✅ (01 Eyl) — ŞU AN BİTTİ
   ↓
3. TAM EĞİTİM 4750/5900 → ~23:45 bitiyor — ŞU AN ÇALIŞIYOR, BEKLE
   ↓ (eğitim bitince)
4. wet_en/paracrawl/caselaw onarımı + enwt dump kararı — Beyin 109M → 40M vektör
   ↓
5. FAISS PQ index (Madde 18) — madencinin RAG'i
   ↓
6. Miner v0.2 Windows (Tauri, imzalı, HF pull, Ollama gibi) — iskelet hazır
   ↓
7. Komuta API + Ödül Faz1/2 — pay→API, Enterprise H×%50
   ↓
8. Debian/Ubuntu port — şirketler için
```
Damıtma merkezde değil, madencide — hazır 32B öğretmen, 1B/3B öğrenci QLoRA ile madenci GPU’sunda.

## 1 YILLIK YOL HARİTASI (10 Eyl 2026)
Tam metin: `docs/YOL-HARITASI-1YIL.md` — Faz 0 testnet kapanış (Eyl-Eki) → Faz 1 mainnet (Kas-Oca) → Faz 2 büyüme (Şub-May) → Faz 3 ölçek (Haz-Eyl 2027). Sıradaki kilit: tohum-0 (DNS+modem, sende) + cc100 embed bitişi.