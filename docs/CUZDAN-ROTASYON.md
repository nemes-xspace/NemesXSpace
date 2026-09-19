# Cüzdan Rotasyon Rehberi (19 Eyl 2026)

> Neden: TRC20 adresi zincirde kalıcı-iz bırakır. Rotasyon, izi kırar.

## Ne zaman döndür

- Her 90 günde bir (rutin), veya
- Adres herhangi bir olayda geçtiyse (log-sızıntısı, ekran-görüntüsü, destek-yazışması).

## Nasıl (kayıpsız)

1. Yeni adres üret (cüzdan-uygulamasında).
2. Yeni adresle `/api/kayit` → yeni token al (eski madenci çalışmaya devam eder — kesinti yok).
3. Yeni madenci ilk payı alınca doğrula (`/api/bakiye`).
4. Eski madenci kaydını bırak (tokenı sil; DB satırı tarihe karışır).
5. Eski adrese bir daha ödeme-talimatı verme.

## Dikkat

- Eski adres zincirde kalır (silinemez) — rotasyon geleceği korur, geçmişi değil.
- Token + cüzdan AYNI ANDA değişir (biri değişip diğeri kalırsa eşleşme karışır).
- Kanarya-kimlik rotasyona dahil DEĞİL (tuzak sabit kalır).
