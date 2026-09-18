# ADR-001: Örümcek Ağı — Yük Mesh'te, Komuta Pusulada

- Tarih: 16 Eyl 2026 (23:59 revizyonu). Statü: kabul (şifre sahibi onayı).
- Bağlam: 30K/1M hesabı tek-komuta varsayımıyla duvara çarpıyordu
  (dağıtıcı ~370/sn tavan, ölçüldü 17 Eyl).
- Karar: kanıt/denetim/parça trafiği madenciler arasında (gossip, relay,
  mesh-denetim, kira); komuta SADECE görev dağıtır + hakemlik eder.
- Alternatifler: (a) büyük-demir komuta — reddedildi (merkezi sunucu
  yasağı, Garanti-6); (b) tam anarşi (kurasız mesh) — reddedildi (ödül
  adaleti için hakem şart).
- Sonuçlar: B12-B21 işleri bu kararın ürünü; federasyon (B16) üst katman.
- Geri-dönüş: karar mimari-omurga; dönüşü yeni ADR + Faz planı gerektirir.
