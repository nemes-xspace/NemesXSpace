# KART PROTOKOLÜ — MicroSD Tören Kasası (A planı: kart + PIN)

> Statü: mekanizma ispatlı (18 Eyl, loop-cihaz E2E). Gerçek kart mühürleme
> OPERATÖRDEDİR (bu araç gerçek sırra dokunmaz).
> Kural: karttaki `.sifre-1024.bin` ve `master-ed25519.pem` ASLA okunmaz,
> kopyalanmaz, taşınmaz — kasa SADECE kilit olarak kullanılır.

## Tehdit modeli (dürüst)

- Kart çalınırsa: LUKS + Argon2id + PIN olmadan açılmaz (kaba kuvvet
  pratik-dışı). PIN 4-128 karakter; 8+ önerilir.
- Kart + PIN birlikte ele geçerse: TAM YETKİ (yapısal risk). Karşılık:
  anında rotation (yeni kasa + yeni PIN + eski autorizasyonların iptali).
- Kötü niyetli okuyucu/host: bu araç hosta güvenir; host şüpheliyse
  kullanılmaz (çevrimdışı makine prosedürü ayrı belgededir).
- CPython notu: yorumlayıcı ara kopyalar bırakabilir (belgeli risk);
  kritik PIN/sır bytearray + mlock + açık sıfırlama ile tutulur.
  Paranoyak katman (Rust v2) açık iştir.
- Takas: bu makinede zram (diske düşmez) — doğrulandı 18 Eyl. Başka
  makinede ÖNCE `swapon --show` ile doğrula; disk-takas varsa mlock ŞART.

## Gerçek kart mühürleme (operatör adımları)

1. Kartı tak, `nemes-kart durum` ile UUID'yi gör (kart_goruldu=true).
2. YEDEK AL: kartın birebir imajı (`dd if=/dev/sdX of=...`) + mühürlü zarf.
   İmaji doğrula (geri okuma + hash). İmaji OLMAYAN karta DOKUNMA.
3. `nemes-kart hazirla --cihaz /dev/sdX --evet` (cihaz adını yaz-onayla),
   PIN'i iki kez gir. (DİKKAT: bu adım kartı SİLER — yedek şart.)
   NOT: Mevcut tören kartı (exFAT + dosyalar) için ÖNCE yedek, SONRA göç;
   göç onayı ayrı verilir (şu an verilmedi).
4. `nemes-kart ac` ile açıldığını doğrula, `kapat` ile kapandığını doğrula.
5. udev kuralını kur (şablon: `nemes-testnet/scripts/99-nemes-kart.rules.sablon`):
   UUID'yi işle, reload et, kartı çıkarıp `kasa_acik=false` olduğunu gör.
6. YEDEK kartı AYNI prosedürle hazırla, FARKLI fiziksel yerde sakla.

## Günlük kullanım

- Hassas işlem: `nemes-kart sarmala --dosya <ad> --env <AD> -- <komut>`
  (ör. push: GH_TOKEN kasadaysa sohbete yapıştırmak bitiyor).
- Kartı çıkarınca kasa kapanır (udev) veya `nemes-kart kapat`.
- PIN 3 kez yanlış → kilitlenme YOK (LUKS saymaz); olağandışı denemeyi
  journal'dan izle (`kasa acilamadi` satırları).

## Rotation (kart kaybı/şüphesi)

1. Yeni kart + yeni PIN ile mühürle (yukarıdaki adımlar).
2. Eski kartın açtığı tüm autorizasyonları iptal et (token revoke listesi).
3. Olayı ALTYAPI-KAYIT'a işle (tarih + kapsam, içerik detayı YOK).
