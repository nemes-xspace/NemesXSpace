# Projeyi Bitirmek İçin Gerekenler — 16 Eyl 2026 (ölçümlü)

> "Bitti" tanımı: mainnet canlı + 100+ madenci + H>$0 + 3+ bağımsız operatör.
> Canlı ölçü: 1.010.754 kanıt, 2 madenci (ikisi de kurucu hostta),
> 50.942 batch, 25 FAISS.

## Tamamlanma cetveli (cephe bazında)

| Cephe | Oran | Gerekçe |
|---|---|---|
| Çekirdek mühendislik (komuta/miner/P2P/FAISS) | ~%75 | Denetim turları canlı, testler var; eksik: CI, integration-üstü yük testi |
| Veri hattı (parse/embed/merge) | ~%85 | cc100+caselaw+newscrawl akıyor; eksik: wet_en/paracrawl зеркала |
| Tokenomik tasarım | ~%90 | Kilitli; eksik: mainnet uygulaması (1.1) |
| Ürünleştirme (miner UX/site/docs) | ~%20 | Linux zip var; yok: Windows, docs sitesi, çeviriler |
| Merkeziyetsizlik | ~%10 | Kod hazır; 0 bağımsız operatör, tohum yarım |
| Hukuk/uyum | ~%5 | Taslaklar var; avukat görüşmesi yok |
| Ekonomi (H, payout, likidite) | ~%5 | H=$0; raylar yok |
| Topluluk | ~%0 | Kanal yok |

## Fazlara göre kalan iş (adam-gün, tek kurucu)

- **Faz 0 (kalan ~4 hafta):** tohum bitirme (3) + operatör kiti (5) + indirme
  provası (3) + çeviriler (6) + bant optimizasyonu (5) + topluluk (3) +
  regresyon CI (4) + telemetri pano (4) + docs sitesi (5) ≈ **38 gün**
- **Faz 1 (~10 hafta):** 1.1 (4) + genesis (3) + Win.exe (10) + multi-sig (4) +
  explorer (6) + bounty (3) + sybil (6) + wallet (8) + hukuk (takvim) +
  prova (5) ≈ **49 gün** (1.4 düştü: -5)
- **Faz 2 (~16 hafta):** payout töreni (4) + DEX (7) + P2P sertleştirme (8) +
  itiraz (5) + corpus-2 (10) + reputation (8) + governance (5) + marketing (8) +
  anti-fraud (6) + denetim (takvim) ≈ **61 gün**
- **Faz 3 (~14 hafta):** on-prem (10) + rapor (5) + işe alım (8) + v2 taslak (4) +
  governance-oy (10) + tatbikat (4) + corpus-3 (10) + partnerlik (6) ≈ **57 gün**
  (3.1/3.2 düştü kapsam kilidiyle: -32)

**Toplam kalan: ~205 adam-gün ≈ 10 ay (5 gün/hafta) → Eyl 2027 hedefi
sıfır boşlukla tutar.** Tek kurucu varsayımıyla tampon YOK.

## Kritik yol (gecikirse her şey kayar)
```
WAF kuralı (2 dk, sen) → dış miner provası → tohum DONE →
bağımsız operatör #1 → mainnet 1.1 (Kas) → genesis → wallet →
H>$0 → Faz 2 kapısı
```
Kod tarafında kritik yol BOŞ — tüm darboğazlar operatör/hukuk/ekonomi masasında.

## En büyük 3 risk
1. **H=$0 sürekliliği:** teknik her şey bitse de ödeyen müşteri yoksa Faz 2
   açılmaz. Panzehir: H motoru (ilk ödeyen profili) Ekim'de adlandırılmalı.
2. **Tek-kişi darboğazı:** senin haftalık bant genişliğin (DNS/WAF kararları
   bile günlerce bekledi). Panzehir: 3.5 işe alım öne çekilsin mi? (v1.2 sorusu)
3. **Mainnet güvenlik borcu:** bağımsız denetim (2.10) + bounty (1.10) olmadan
   gerçek para riske girer. Takvimden çıkarılamaz.

## Önerilen 30 gün (sıralı)
1. Hafta: WAF + dış prova + tohum DONE + operatör kiti v1.
2. Hafta: site Y-serisi + çeviriler + docs sitesi iskeleti.
3. Hafta: regresyon CI + telemetri pano + bant optimizasyonu.
4. Hafta: avukat randevusu + 1.1 prova planı + Faz 0 kapanış ölçümü.
