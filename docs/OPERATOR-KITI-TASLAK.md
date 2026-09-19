# Operatör Kiti v1 — TASLAK (2. düğüm kurulumu, onay bekliyor)

> Hedef (0.4): teknik bilgisi temel düzeyde bir gönüllü, ev makinesinde
> komuta düğümünü 1 saatte ayağa kaldırır. Hiçbir adımda kurucuya soru sormaz.

## 0. Gereksinimler
- Linux (Ubuntu 22.04+), 4 CPU / 8G RAM / 100G boş disk, ev interneti.
- GPU ŞART DEĞİL (komuta düğümü CPU ile çalışır; GPU madencilerde olur).
- Cloudflare hesabı (ücretsiz katman yeter).

## 1. Kurulum (5 komut)
```bash
# 1) Paketi indir + doğrula (iki-katman: hash + imza; TOFU-değil)
curl -O https://nemes-x.space/miner/nemes-miner-v0.2.0-linux.zip
sha256sum -c SHA256SUMS
# + SHA256SUMS.sig dogrulamasi (README.txt'deki anahtar+komutla)
# 2) Komuta binary'si (DUYURUDAKİ sürüm ve hash ile)
# 3) Dizin + servis dosyaları kopyala (paketteki systemd/ klasörü)
sudo cp systemd/* /etc/systemd/system/ && sudo systemctl daemon-reload
# 4) Tünel: cloudflared login + hazır config'i kendi domainine uyarla
# 5) Başlat + doğrula
sudo systemctl enable --now nemes-komuta
curl -s http://127.0.0.1:8787/health
```

## 2. Doğrulama kontrol listesi (operatör işaretler)
- [ ] `/health` 200 dönüyor.
- [ ] `journalctl -u nemes-komuta` hatasız 5 dk.
- [ ] Tünel üzerinden dış curl 200 (kurucuyla prova).
- [ ] Yedek timer aktif (`komuta-backup.timer`).
- [ ] Disk eşiği izlemede (izleme.py kopyası + timer).

## 3. Sorumluluklar (kısa)
- Makine açık kalır; kesintide ceza YOK (ağ tolere eder).
- Anahtar/token dosyaları (600) kimseyle paylaşılmaz.
- Şüpheli log → kurucuya iletilir (kanal: duyurulacak).

## 4. Paketlenmesi gerekenler (hazırlayan: ben)
- [ ] komuta-rs binary + SHA256
- [ ] systemd unit seti (komuta + backup timer + izleme)
- [ ] Caddyfile şablonu + cloudflared config şablonu
- [ ] izleme.py kopyası
- [ ] Bu kılavuzun son hali

## Açık sorular (operatör cevaplar)
- Dağıtım kanalı ne? (site indirme sayfası / GitHub releases)
- İlk gönüllü kim? (0.4 kapanışı: 1 gönüllüde 2. düğüm)
