# Tohum-0 Kurulum Checklist — operatör (ev makinesi)

> Hedef (0.3): `komuta.` DNS + TLS + dışarıdan kayıt→görev→kanıt turu yeşil.
> Kod tarafı hazır (P2P duyuru, komuta-rs, miner). Aşağıdakiler SENDE.

## A. DNS (alan adı paneli)
- [ ] `komuta.nemes-x.space` → ev dış IP'sine A kaydı (TTL 300).
- [ ] Doğrulama: `dig +short komuta.nemes-x.space` ev IP'sini dönmeli.

## B. Modem (port yönlendirme)
- [ ] 80/tcp → komuta makinesi (TLS onayı için).
- [ ] 443/tcp → komuta makinesi (Caddy TLS bitirir).
- [ ] 4003/tcp → komuta makinesi (P2P mesh).
- [ ] 8787 DIŞA AÇILMAZ (localhost-only; dış erişim Caddy üzerinden).
- [ ] Doğrulama (dış ağdan, örn. telefon): `curl -s https://komuta.nemes-x.space/api/arz | head -c 60`.

## C. Makine (benim provamla birlikte)
- [ ] Caddyfile'a `komuta.nemes-x.space` bloğu (TLS otomatik).
- [ ] `nemes-komuta` + `caddy` restart, journal temiz.
- [ ] Dış prova turu: kayıt → görev → kanıt → status (ben çalıştırırım, sen izlersin).

## D. Güvenlik notları
- [ ] Modem yönetici şifresi varsayılan DEĞİL.
- [ ] UPnP kapalı (elle yönlendirme esastır).
- [ ] Elektrik kesintisi sonrası servisler otomatik (enabled Birimler: komuta/miner/caddy/tunnel).

## Kabul kriteri
Dışarıdan 1 madenci kaydı + ilk payı alır → 0.3 DONE, Faz 0→1 kapısı için sayılır.
