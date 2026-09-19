# Felaket Provası — Tüzük, Senaryolar, Ölçümler (19 Eyl 2026)

> Tüzük: izole kopya, sentetik/bozuk-veri serbest, üretim DB'ye yazma YOK,
> geri-dönüş hazır, dersler deftere (IR-2026-004/005, RR-2026-002).

## BÖLÜM 1 — Tüzük (uygulanan)

Tatbikatlar kopya üzerinde icra edildi; üretim etkilenmedi (8787 tüm
süreçte sağlıklı). Gözlemci: otomatik log + zaman ölçümü.

## BÖLÜM 2–3 — Senaryo kataloğu + kurtarma (icra edilenler ✅)

| # | Senaryo | İcra | Sonuç |
|---|---|---|---|
| T1 | DB kaybı → yedekten dön | Kopya 12sn + integrity ok + izole-boot | ✅ CANLI açıldı, arz sundu |
| T2 | Bozuk yedek tespiti | Başlık sıfırlanmış kopya | ✅ anında `not-a-database (26)` |
| T3 | Binary geri-dönüş | `.bak-18eyl` duman testi | ✅ miner `--version`, komuta DB-yok hatasıyla (beklenen) çıkış |
| T4 | Restart RTO ölçümü | 18 Eyl gerçek deploy günlüğü | ✅ stop→servis ~2-3 dk |

İcra edilmeyenler (kâğıt-üstü, sebep): disk-doldurma (üretimi öldürür),
host-kaybı (ikinci makine yok), ekip-kaybı (masa-başı planı var,
provası yok), DDoS (üçüncü-tarafa dokunulmaz).

## BÖLÜM 4 — Ölçüm tablosu

| Senaryo | RTO hedef* | RTO ölçülen | RPO hedef* | RPO ölçülen | MTTD | MTTR | Fark/neden |
|---|---|---|---|---|---|---|---|
| DB kaybı | 30 dk | ~3 dk (12sn kopya + ~2dk boot) | 24 sa | ~19 sa (günlük yedek) | otomatik (timer) | ~3 dk | HEDEF İÇİNDE ✅ |
| Bozuk yedek | — | anında (integrity) | — | — | otomatik | 0 (red) | ✅ |
| Binary geri-dönüş | 30 dk | <1 dk (cp + start) | — | — | insan | <5 dk | ✅ (prova: duman testi) |
| Host kaybı | tanımsız | ÖLÇÜLMEDİ | tanımsız | — | — | — | ❌ hedef + prova yok |
| Ekip kaybı | tanımsız | ÖLÇÜLMEDİ | — | — | — | — | ❌ |

\* Hedefler ÖNERİ (onaylı değil): RTO 30dk/RPO 24sa. Onay operatörde.

## BÖLÜM 5 — Yürütme protokolü

Ön-bilgi verilmedi (gerçekçi) → izole icra → gözlem (süreli) →
müdahale-yok-gerekmedi → sıcak-değerlendirme (bu dosya) →
deftere işlendi → 3 ay sonra tekrar.

## BÖLÜM 6 — İletişim planı

- İç: tek kişi — kriz masası = operatörün kendisi; karar mercii hak-sahibi.
- Dış: madencilere duyuru kanalı YOK (eksik! site/blog mu, gossip `nemes/komut` mu? karar gerekli).
- Kural: doğrulanmamış bilgi paylaşılmaz; her mesaj deftere.

## BÖLÜM 7 — Dersler

1. `/tmp` tmpfs 16G — 8G kopya doldurur; kopyalar diske (`yedek/` dışı
   geçici adla) alınacak. (Tatbikatta yakalandı, üretim etkilenmedi.)
2. Restore KANITLANDI — runbook adım 1'in "önce kopyada dene" cümlesi
   artık ölçülü prosedür (12sn + 2dk).
3. RTO/RPO hedefleri resmen onaylanmadı — onaylanmadan "söz" yok.

## BÖLÜM 8 — Yönetici özeti

- En kritik 5 senaryo: host-kaybı, ekip-kaybı, disk-dolması, DDoS, bozuk-yedek-zinciri.
- En zayıf 3 kurtarma: host (yok), ekip (yok), dış-duyuru (yok).
- En acil 3 eylem: (1) RTO/RPO onayı, (2) madenci-duyuru kanalı kararı, (3) 2. lokasyon yedek kopyası.
