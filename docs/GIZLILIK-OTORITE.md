# Otorite Gizliliği — Durum ve Kurallar (Prompt 16-revize, 19 Eyl 2026)

> İlke: kontrollü görünürlük. AI karar vermez (AI-SINIRLARI.md geçerli).

## Ölçülen gizlilik duruşu

| Öğe | Durum | Kanıt |
|---|---|---|
| Kimlik (gerçek) | GİZLİ (bilinen-kayıt yok) | OSINT taramasında geçmiyor |
| Kamu kimliği | `nemes-xspace` org + X hesabı (temsil) | public |
| Konum | Origin CF-arkası (172.67.201.161 proxy) | DNS ölçümü |
| Şifre-değeri | Bende YOK (3 ret kayıtlı) | defter |
| Halef | ATANMADI (gizli tutulacak şey yok henüz) | OTORITE-HIYERARSI |
| Switch eşiği | ENV'e alındı (`OTORITE_NABIZ_GUN`, varsayılan 14) | izleme.py (bu tur) |
| Nabız dosyası | local-only, 600 | stat |

## Yeni bulgu: git-geçmişi ifşası (GIZ-01)

Public commitler: saat-dilimi (+03), dil (Türkçe), tek-kişi ritmi,
çalışma-saatleri deseni. Bunlar geri alınamaz (push'lu tarih).
Karar: KABUL (yazılı, gerekçeli: geliştirme-şeffaflığı > desen-gizliliği;
kritik-sır içermez — token/şifre/ip YOK, taranmış). Gözden-geçirme: 6 ayda bir.
Gelecek: hassas-zamanlı commitlerden kaçınmak (gece-toplu-push) — öneri,
zorunluluk değil.

## AI rolü (tekrar-tescil)

İz-küçültme kararı: SAHİP. AI: öneri + onaylı-icra. Bu turdaki tek
eylem (ENV-eşiği) onaylı-kapsamdaydı (sürüm-kilidi görevi).

## Yanıltıcı-iz

ÜRETİLMEDİ (Prompt 7 kararı sürdürülüyor: gerekçe yok).
