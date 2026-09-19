# Otorite Hiyerarşisi + Yetki Devri (Prompt 13, 19 Eyl 2026)

> Üst-hüküm: `docs/MUTLAK-OTORITE.md` (berat). Bu dosya operasyonel ektir;
> çelişmede berat kazanır. Şifreye dair DEĞER içermez (konum/prosedür
> anlatır, sırrın kendisini asla yazmaz).

## Katmanlar (bugünkü doluluk)

| Katman | Rol | Kim | Yetki | Durum |
|---|---|---|---|---|
| K0 | Tek Otorite | sahip (beratlı) | mutlak | AKTİF |
| K1 | Yardımcı otorite | BOŞ (atanmadı) | — | AÇIK (görev) |
| K2/K3 | Yönetici/Operatör | sahip (aynı kişi) | mutlak (K0'dan) | AKTİF |
| K4 | Madenciler | 3 token-kimliği | kendi-işi | AKTİF |
| K5 | Dış (CF/GitHub/Zoho) | sözleşmesiz-hesap | hizmet | AKTİF |

## Yetki devri prosedürü (K0 → K1)

1. Otorite yazılı devreder (kapsam + süre + `denetim/kayitlar.jsonl`'e DR).
2. Devralan ayrı kimlikle çalışır (K0 şifresi ASLA paylaşılmaz — berat m.9).
3. Süre bitince yetki düşer; kullanım logu gözden geçirilir.
4. Acil-geri-alma: K0 tek cümleyle iptal eder (kayıt şart).

## Nabız (dead-man's switch — hafif sürüm)

- Dosya: `nemes-testnet/otorite-nabiz` (600). Sahip periyodik `touch` eder.
- Eşik: 14 gün sessizlik → izleme ALARM üretir (canlı-doğrulanmış).
- Yanlış-alarm koruması: alarm = dikkat, kilitleme YOK (otomatik-kilit
  reddedildi: tek-kişilik sistemde kilit = kendini-DoS).
- Tam switch (otomatik-kilit/halef-çağrı): tasarım-aşamasında, karar sahiple.

## Yokluk senaryoları (durum)

- Geçici: K1 BOŞ olduğu için kritik işlem bekler (kayıtlı-kabul, süreli:
  K1 atamasına kadar).
- Kalıcı: vasiyet-mektubu prosedürü (`VASIYET-SABLON.md`) — mektup
  YAZILMADI (en acil insan-eylemi, sahipte).
- Zorla: prosedür yok (görev).
- İhanet: çift-kayıt yok (tek kişi; bağımsız-denetçi atanmadı — görev).
