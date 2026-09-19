# ADR-004: Sürüm-Kilidi API Kararları (19 Eyl 2026)

> Bağlam: anonim pahalı-uç + sınırsız açık-kayıt (RT-001/FR-007/008).
> Karar verici: hak-sahibi (onay: sohbet) + denetçi (uygulama).

## Kararlar

1. `/api/ara` token-kapılı. Ödün: anonim demo öldü; kazanç: DoS/amplifikasyon
   kapandı. Kayıt açık-ücretsiz olduğu için erişim korunur.
2. Kayıt kovası 10/sa/IP (CF-Connecting-IP > XFF > peer). Ödün: NAT-arkası
   paylaşımlı IP'de 11. kullanıcı bekler; kazanç: Sybil maliyeti.
3. CORS allowlist (site+Tauri+localhost). Ödün: yeni istemci menşei kod
   değişikliği ister; kazanç: tarayıcı-tabanlı istismar yüzeyi kapandı.
4. Gövde limiti 512KB. Ödün: 100'lük toplu-kanıt ~100KB'da rahat; kazanç:
   şişirme-yükü sınırlı.
5. Toplu-kanıt 401-önce. Ödün: yok (davranış tekille eşitlendi).

## Reddedilenler

- Kayıt CAPTCHA'sı: bot-ticareti yok, madencide tarayıcı yok; kova yeterli.
- Global rate-limit katmanı: tek-komuta testnet'te kova yeterli; mainnet
  gateway'inde tekrar değerlendirilecek.
- `/api/ara` tam-kapatma: kayıtlı-kullanıcı demo değeri korunmak istendi.

## Sonuçlar

27/27 test (kova/401 testleri dahil), canlı-doğrulama (401/422/429
yolları + http-red logları). Geri-dönüş: `.bak4` binary'leri.
