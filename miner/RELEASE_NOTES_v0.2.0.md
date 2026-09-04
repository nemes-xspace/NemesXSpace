# NEMES Miner v0.2.0 — Windows (Tauri) — Plan v2

> **Durum:** iskelet hazır, build için Rust gerekiyor (cargo yok, CI’da build edilecek)

## Yenilikler
- Tauri iskelet (12MB NSIS, imzalı — karar 2)
- Ollama benzeri HF pull — önerilen 4 model + serbest arama (karar 3)
- Hem mining hem local chat (karar 4)
- Komuta API: /srv/beyin/kaynaklar/komuta_api.py

## Önerilen Modeller
| Model | Boyut | VRAM |
|---|---|---|
| Qwen2.5-3B Q4_K_M (varsayılan) | 2.0GB | 4GB+ |
| Gemma-2-2B Q4 | 1.6GB | 3.5GB+ |
| Qwen2.5-7B Q4_K_M | 4.5GB | 6GB+ |
| Qwen2.5-14B Q4_K_M | 8.5GB | 10GB+ |

## Kurulum (CI sonrası)
- `nemes-miner_0.2.0_x64-setup.exe` (imzalı NSIS)
- SHA256 + pubkey ile doğrula
- İlk açılış: HF’den model seç → pull → mining başla

## Komuta
- Heartbeat 45sn (Md.113) — merkez olmadan kilitlenir
- API: 8787 port, `uvicorn komuta_api:app`

## Build
```bash
# CI’da
npm install
npm run tauri build
# + signtool
```

## Sonraki
- Debian .deb port (aynı Rust core)
- FAISS RAG bağla
