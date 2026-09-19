# Veri-Sahibi Talep Prosedürü (Taslak — hukukçu onayı YOK, 19 Eyl 2026)

> Kapsam: cüzdan-adresi + token-meta (tek kişisel-nitelikli veri).
> Kanal: `nemes-x.space@zohomail.eu` ([PRIVACY] konulu) veya GitHub issue.
> Sahiplik: madenci, cüzdan-imzasıyla kendini kanıtlar (TRC20 adresinden
> imzalı mesaj — taklit talepleri elenir).

## Akış (30 gün)

1. AL (0-3 gün): talep deftere işlenir (tarih + kanal + istenen hak).
2. DOĞRULA (3-7 gün): cüzdan-imzası istenir; doğrulanmazsa RED (gerekçeli).
3. YERİNE-GETİR (7-25 gün):
   - Erişim: ilgili satırlar JSON olarak verilir.
   - Silme: token iptal (kara-liste) + cüzdan maskeleme; kanıt/ledger
     satırları SİLİNMEZ (ağ-muhasebe bütünlüğü — gerekçe yazılı bildirilir).
   - Taşıma: JSON dışa-aktarım.
   - İtiraz: insan-incelemesi (sahip).
4. KAPAT (25-30 gün): yanıt + defter-kaydı. Süre-aşımı = ihlal (alarm).

## Sınırlar (dürüst)

- Anonim kurucu + tüzel-kimliksizlik: resmi merci muhatabı tanımsız (S7'ye bağlı).
- Ledger değişmezliği vs silme-hakkı ÇELİŞİR — çözüm: maskeleme (içerik
  durur, kimlik bağı kopar). Hukukçu onaylamadan nihai değildir.
