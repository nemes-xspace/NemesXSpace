# Komuta Operatörü Kılavuzu — Herkes Komuta Çalıştırır (VPS'siz Ağ)

> İlke: merkezi sunucu yoktur. Komuta açık binary'dir; çalıştıran düğüm olur.
> Madenci `--komuta` ile istediğine bağlanır; mesh gossip ile dillenir.
> Bu belge ev makinesinde düğüm açmayı anlatır (10 dakika + modem ayarı).

## 0. Cloudflare Kalkanı (ŞART — ücretsiz, 15 dakika)

Ev IP'n asla dünyaya açık görünmez. Trafik Cloudflare kenarında süzülür.

1. **Hesap:** cloudflare.com → ücretsiz kaydol → "Add domain" → `nemes-x.space` yaz.
2. **Kayıtları kopyala:** Cloudflare DNS'i otomatik tarar — GitHub Pages
   kayıtların dahil her şeyi aynen aldığını KONTROL ET (eksik varsa elle ekle).
3. **Nameserver değiştir:** domain panelinde (Hostinger: DNS/Nameservers)
   mevcut `*.dns-parking.com` ikilisini Cloudflare'in verdiği ikiliyle değiştir.
   Yayılma 5 dk – 24 saat.
4. **komuta kaydı:** Cloudflare DNS → Add record → A → isim `komuta`,
   içerik ev IP'n → **Proxy AÇIK** (turuncu bulut). Bu, IP'ni gizler.
5. **SSL:** SSL/TLS → Full (Strict). Caddy otomatik sertifika alır (aşağıda).
6. **WAF:** Security → WAF → kural ekle: `/api/*` → 100 istek/dk/IP üstü
   engelle. "Under Attack Mode" kapalı dursun — saldırı anında tek tıkla açılır.

Doğrulama: `curl https://komuta.seninadın.org/api/saglik` (operatöre sorulur).

## 0. Gerekenler

- Linux x86_64 (ev PC/sunucu), 4GB+ RAM, 50GB+ disk
- Herkese açık IP (dinamik olur — §4) + modemde port yönlendirme yetkisi
- Alan adı (ör. `komuta.seninadın.org`) veya bizim tohum listesine kayıt

## 1. Kurulum (5 komut)

```sh
mkdir -p ~/nemes-public/bin && cd ~/nemes-public
# binary'yi siteden indir, SHA256 doğrula:
unzip nemes-komuta-v*.zip && chmod +x bin/komuta-rs
# ilk DB oluşur, migration'lar otomatik uygulanır (sadece CREATE):
PORT=8787 P2P_PORT=4003 GOREV_CORPUS=tr ./bin/komuta-rs
```

## 2. Servisleştirme

`komuta-rs/deploy/nemes-komuta-public.service` dosyasındaki YOLLARI kendi
kullanıcına göre düzenle, `/etc/systemd/system/` altına koy:

```sh
sudo systemctl enable --now nemes-komuta-public
journalctl -u nemes-komuta-public -f   # log izle
```

## 3. TLS (Caddy, otomatik sertifika)

```sh
sudo apt install caddy   # veya https://caddyserver.com/download
# komuta-rs/deploy/Caddyfile içindeki alan adını yaz:
sudo cp Caddyfile /etc/caddy/Caddyfile && sudo systemctl reload caddy
```

## 4. Ev interneti notları (dürüst bölüm)

- **Portlar:** modemden 80+443 (TLS) ve 4003 (P2P mesh) dışarı açılmalı.
  Açılmazsa düğümün HTTP'ye cevap verir ama mesh'e katılamaz (yarım düğüm).
- **Dinamik IP:** değişirse DNS'i güncelle (çoğu modemde DDNS/No-IP
  desteği var; 5 dk ayar). Tohum listesi IP değil **alan adı** tutar.
- **Kesinti:** fiş çekilirse madenciler başka düğüme kayar (ceza yok).
  24 saatten uzun kapalı kalınacaksa tohum listesinden geçici çıkarılmayı
  bildir (mail atman yeterli).

## 5. Tohum listesi (DNS seeds)

- `komuta.nemes-x.space` — topluluk düğümlerinin alan adları (DNS çoklu
  A kaydı; tek sunucu değil, listedir).
- Listeye girmek: düğümün 7 gün kesintisiz çalışsın, mail at
  (nemes-x.space@zohomail.eu), adresin listeye eklenir.
- Miner tarafı hazır: `nemes-miner mine --komuta <SENİN ADRESİN>`.

## 6. Operatör kuralları (kısa)

1. DB'ye elle ALTER/DROP yok (sadece yedek al).
2. Log'larda metin paylaşma (kör ID kuralı seni de bağlar).
3. Kara liste kararları sana ait düğümünde geçerlidir (ağ geneli için
   imha delili + imza gerekir).
4. Sürüm duyurularını takip et (güvenlik yaması çıkarsa 48 saatte geç).
