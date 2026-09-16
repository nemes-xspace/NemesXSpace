# Tohum-0 Kurulum Checklist — operatör (ev makinesi)

> Hedef (0.3): `komuta.` DNS + TLS + dışarıdan kayıt→görev→kanıt turu yeşil.
> Durum 16 Eyl: **HTTP(S) bacağı CANLI** — Cloudflare Tunnel
> (`komuta.nemes-x.space` → 127.0.0.1:80 → Caddy → 127.0.0.1:8787),
> DNS Cloudflare'de, dış prova 200 döndü. **Modem ayarı GEREKMİYOR (HTTP için).**

## A. DNS — ✅ YAPILDI (Cloudflare)
- `komuta.nemes-x.space` Cloudflare edge'e bağlı (tunnel route).
- Doğrulandı: dig CF IP'leri + dış curl 200.

## B. Cloudflare paneli — SENDE (2 dk)
- [ ] WAF → `komuta.nemes-x.space/api/*` için **Skip kuralı** (Bot Fight Mode
  çıplak API istemcilerini 1010 ile kesiyor; 16 Eyl kanıtlandı).
  Kural yoksa dış miner'lar kayıt olamaz. Miner UA (`NEMES-Miner/0.2`)
  ikinci savunmadır, yerine geçmez.
- [ ] TLS modu: Full (Strict) önerilir (Caddy'de otomatik sertifika var).

## C. Modem — SADECE P2P İÇİN (mesh'e tam katılım istenirse)
- [ ] 4003/tcp → komuta makinesi (tünel ham TCP taşımaz).
- [ ] 80/443/8787 AÇILMAZ (tünel + localhost-only yeterli).
- [ ] UPnP kapalı, modem şifresi varsayılan değil.

## D. Dış prova turu (benimle birlikte)
- [x] Dış curl 200 (16 Eyl).
- [x] Dış kayıt 200 (UA ile, 16 Eyl; test satırı temizlendi).
- [ ] Gerçek dış makineden miner kaydı + ilk pay (WAF kuralından sonra).

## Kabul kriteri
Dışarıdan 1 madenci kaydı + ilk payı alır → 0.3 DONE, Faz 0→1 kapısı için sayılır.
