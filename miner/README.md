# NEMES Miner — Windows (Tauri)

> Mine knowledge. Not hashes. — Komuta sizde, güç madencide.

## Mimari
- **Tauri (Rust + WebView)** — 12MB installer, NSIS imzalı
- **Ollama benzeri UX** — `pull` ile HF’den GGUF indir, lokalde çalıştır
- **Komuta merkezi** `/srv/beyin` — heartbeat 45sn, merkez olmadan kuş uçmaz (Md.113)

## Önerilen Modeller (HF GGUF)
| Model | Boyut Q4_K_M | VRAM | Açıklama |
|---|---|---|---|
| `bartowski/Qwen2.5-3B-Instruct-GGUF` | ~2.0GB | 4GB+ | **Varsayılan** — Türkçe en iyi küçük model |
| `bartowski/gemma-2-2b-it-GGUF` | ~1.6GB | 3.5GB+ | Ultra hafif — 4GB GPU’lar |
| `bartowski/Qwen2.5-7B-Instruct-GGUF` | ~4.5GB | 6GB+ | Dengeli |
| `bartowski/Qwen2.5-14B-Instruct-GGUF` | ~8.5GB | 10GB+ | Güçlü — mini öğretmen |

Kullanıcı serbest arama ile HF’den istediği GGUF’u çekebilir (`hf-hub` resume’li).

## Geliştirme
```bash
# Rust kurulumu (bir kez)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Node deps
npm install
npm run tauri dev      # dev
npm run tauri build    # NSIS imzalı installer (signtool gerektirir)
```

## Modüller
- `src-tauri/src/heartbeat.rs` — 45sn komuta kalp atışı, Ed25519 doğrulama
- `src-tauri/src/models.rs` — HF pull, ~/.nemes/models
- `src-tauri/src/worker.rs` — embed worker, int8 nicele (wiki_embed_par.py:30 ile uyumlu)
- `src-tauri/src/komuta.rs` — API istemcisi

## Komuta API
Merkez: `/srv/beyin/kaynaklar/komuta_api.py`
- `POST /api/kayit` `POST /api/heartbeat` `GET /api/gorev` `POST /api/kanit` `GET /api/rag`

## Kurulum
1. `nemes-miner_0.1.0_x64-setup.exe` indir (imzalı)
2. Cüzdan bağla → token.json şifreli
3. Model yükle → Mining başla
