# Veri Envanteri + Sınıflandırma (Prompt 11, 19 Eyl 2026)

> Şema: KAMU / DAHİLİ / GİZLİ / ÇOK GİZLİ. Varsayılan DAHİLİ;
> PII/PCI/sağlık verisi YOK (kamusal corpus + operasyonel veri).

| ID | Varlık | Sınıf | Sahibi | Saklayan | Erişen | Süre | İmha | Şifre | Log |
|---|---|---|---|---|---|---|---|---|---|
| VD-01 | kanitlar (vektör+madde_id) | DAHİLİ | ağ | komuta.db+yedek | miner/komuta | süresiz (iş) | — | yok (açık) | kısmi |
| VD-02 | miners.token | ÇOK GİZLİ | madenci | komuta.db+yedek (düz-metin!) | komuta | hesap-ömrü | B39 sonrası hash | YOK | yok |
| VD-03 | miners.cuzdan (TRC20) | GİZLİ | madenci | komuta.db+yedek | komuta | hesap-ömrü | hesap-kapanış | yok | yok |
| VD-04 | ledger (muhasebe) | GİZLİ | ağ | komuta.db+yedek | komuta | süresiz (denetim) | — | yok | — |
| VD-05 | escrow (bekleyen pay) | DAHİLİ | ağ | komuta.db | komuta | sonuçlanana-kadar | otomatik (serbest/yanma) | yok | — |
| VD-06 | FAISS index (türev-vektör) | DAHİLİ | ağ | /srv/beyin/wiki | komuta/miner | süresiz | dosya-silme | yok | — |
| VD-07 | wiki ham-metin (kamusal) | KAMU-türev | — | wiki_*.db (salt-okunur) | komuta | süresiz | — | yok | — |
| VD-08 | master-ed25519 (kart) | ÇOK GİZLİ | sahip | fiziksel kart+PIN | sahip | süresiz | kart-imha | PIN | tören |
| VD-09 | miner-ed25519 (imza anahtarı) | GİZLİ | madenci | ~/.nemes/*.key (600) | miner | süresiz | dosya-silme | yok | — |
| VD-10 | yedekler (VD-01..05 kopyası) | EN-YÜKSEK-SINIF (ÇOK GİZLİ) | ağ | yedek/ (600, 9 kopya) | operatör | 7+manuel | rotasyon-silme | YOK (açık!) | — |
| VD-11 | journal/uygulama-logu | DAHİLİ | — | journald + izleme-log | operatör | varsayılan (sınırsız!) | logrotate-yok | yok | — |
| VD-12 | kanarya-credential | ÇOK GİZLİ | denetçi | komuta.db (satır) | kimse (tuzak) | süresiz | — | yok | alarm-var |

## Bütünlük kararları (19 Eyl, ölçüldü)

- Yetim-kanıt: 0 ✅. Defter/coin kayması: 395 mikro (tarihsel, 4 Eyl).
- Kök: 8 coin+ledger çifti transactionsuzdu → **odeme_yaz() atomik
  helper'a çevrildi** (27/27 test). Tarihsel 395'e dokunulmadı
  (muhasebe-bütünlüğü: geçmiş yeniden-yazılmaz, not düşülür).
- Log-sızıntı taraması: kod-loglarında token yok; journal'da 32-hex yok ✅.

## Saklama kararları (öneri → sahip onayı bekler)

- kanitlar/ledger: süresiz (ağ-muhasebesi; silme = güven-kaybı).
- Token: hesap-kapanışında silme (prosedür yazılmadı — görev).
- Log: 90 gün (journald cap + logrotate görevi açık).
- Yedek: 7 günlük + manuel-kilitli (aktif).

## Açıklar (kabul-değil, görev)

- Duruşta-şifreleme YOK (ext4/LUKS-suz, ölçüldü) → seçenekler:
  (a) LUKS geçişi (kesinti+anahtar-yönetimi), (b) SQLCipher (uygulama),
  (c) fiziksel-güvenlik kabulü (yazılı). Karar sahipte.
- Yedek-şifreleme YOK → (a) ile birlikte veya gpg-simetrik katman.
- Token-hash (B39) VD-02/VD-10 riskini düşürür (bağımlı).
