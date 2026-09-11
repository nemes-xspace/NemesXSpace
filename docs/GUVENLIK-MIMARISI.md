# NEMES-X Güvenlik Mimarisi — Kapalı Ören Ağ

> Sürüm 1.0 — 09 Eyl 2026. İlke: **madenci parçayı görür, resmi asla göremez;
> sızdıran kanıtıyla imha edilir.** Fiş çekmek serbest, hainlik yasak.

## 0. Tehdit Modeli

| Aktör | Yeteneği | Hedefimiz |
|---|---|---|
| Meraklı madenci | Kendi batch'lerini okur | Batch'ler bağlamsız + kör ID'li → işe yaramaz |
| Büyük firma (çok düğüm) | Birçok batch toplar | Rotasyon + bölge yasağı → resim birleşmez |
| Dış dinleyici | Teli dinler | Noise/HTTPS + 24s anahtar rotasyonu |
| Sahte düğüm | Ağa girmeye çalışır | Ed25519 TOFU + ilk shard ilanı denetimi |
| İç hain (kanıtlı) | Veriyi dışarı taşır | Kanarya → ban + pay sıfırlama + kara liste |

**Kabul:** Embed için plaintext şart (FHE/TEE 2026 ev PC'sinde pratik değil).
Savunma "görmemesi" değil, **"gördüğüyle bir şey yapamaması + yakalanması"** üstüne.

## 1. Parçala-Körleştir

- Görev = rastgele ~20 metinlik kör batch. Sıra bilgisi yok.
- Corpus adı tellerde **hash kod** (`tr` → `c9f2` gibi, komuta içi sözlükte çözülür).
- Madde ID'leri her dağıtımda **tek kullanımlık tuzlanmış hash**
  (`blake3(gunluk_tuz + gorev_id + madde_id)`). Miner corpus pozisyonunu
  haritalayamaz, iki batch'i birleştiremez.
- Rotasyon: aynı miner aynı bölgeyi üst üste alamaz (cursor + claim atlama +
  son-100-görev bölge kaydı).

## 2. Taşıma + Kimlik

- P2P: TCP + Noise + Yamux (mevcut). HTTPS API (mevcut). Ed25519 imza + TOFU (mevcut).
- **Ek:** oturum anahtarı 24 saatte bir yenilenir; eski anahtarla gelen istek düşer.
- Miner'lar arası doğrudan veri kanalı **yasak**: sadece gossip duyuru
  (`nemes/gorev`, `nemes/shard`) + komuta relay. Mesh = ilan tahtası.

## 3. Kanarya Tuzakları (sızıntı dedektörü)

- Komuta her gün N göreve sentetik filigranlı cümle serpiştirir (dışarıda
  indexlenmemiş, kime gittiği `kanaryalar` tablosunda kayıtlı).
- Tarayıcı (günlük cron): bu cümleler web'de / rakip datasette / model
  çıktısında görülürse kaynak miner bellidir.
- Eşik: 1 kanarya = şüpheli (yoğun denetim), 2 kanarya = **kanıtlı** → imha.

## 4. İmha (keseni yok etme)

- Kanıtlı sızıntı/hile → **ban + birikmiş pay sıfırlama + pubkey kara listesi**.
- Mevcut 3-strike (kalite) aynen durur; bu ayrı pisttir (sadakat).
- Fiş çekme / yavaşlık / hata **cezasızdır** — imha sadece kanıtlı hainliğe.
- İtiraz: kara liste operatör onayıyla kalkar (affet komutu mevcut).

## 5. Kilit-Taşı (komuta tarafı)

- Görev DB'si şifreli diskte (LUKS, kurulum belgesi §6). Yedekler şifreli.
- Log'larda **asla düz metin yok**: sadece `gorev_id`, hash önekleri, sayılar.
- 70B ağırlıklar + birleşmiş index kapalı kalır (mevcut kural).

## 6. DB Şifreleme Kurulumu (ops)

```sh
# /srv/beyin şifreli değilse (kurulumda bir kez):
sudo cryptsetup luksFormat /dev/sdb1   # DİKKAT: veri siler, boş diske!
sudo cryptsetup open /dev/sdb1 beyin_crypt
# /etc/crypttab + /etc/fstab girdileri, anahtar dosyası 0400 root'ta.
# Mevcut dolu diskte: yeni şifreli diske rsync ile taşıma planı gerekir.
```

> **Not:** Mevcut `/srv/beyin` dolu ve LUKS'suz. Şifreleme = yeni disk +
> taşıma işi. Kod tarafı (1-4) diskten bağımsız, önce o biter.

## 7. Kod Haritası

| # | İş | Dosya | Durum |
|---|---|---|---|
| 1 | Kör ID (rastgele kor + esleme) | komuta `gorev_kaydet`/kanit/denetim | ✅ kodlandı |
| 2 | Corpus hash kodu + log temizliği | komuta + miner (metin yok, doğrulandı) | ✅ kodlandı |
| 3 | Kanarya ekici (%4) + `/api/kanarya/kontrol` + 5 tohum | komuta içi (ayrı modüle gerek kalmadı) | ✅ kodlandı |
| 4 | İmha (`imha` komutu: ban+pay sıfır) + kara liste kapıları | komut + gorev + kanit | ✅ kodlandı |

*Son güncelleme: 09 Eyl 2026.*
