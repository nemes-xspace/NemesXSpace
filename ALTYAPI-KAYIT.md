# NEMES-X ALTYAPI KAYIT DOSYASI
> Tek hakikat kaynağı (teknik altyapı için). Proje vizyonu için `00-PROJE-TANIMI.md`
> Ödül stratejisi için `docs/TOKENOMI.md` geçerlidir.
> Oluşturuldu: 2026-09-06 (altyapı elden geçirme sonrası, build modunda icra edildi)

## 0. Garanti kuralları (değişikliklerde uyulur)
1. Önce kaydet, sonra kes — bu dosya her fazda güncellenir.
2. Eğitim sürerken GPU'ya dokunulmaz; gerekmedikçe servis restart edilmez.
3. Canlı DB'ye `ALTER/DROP` yok — sadece `CREATE TABLE` ve `ADD COLUMN`.
4. Her migration öncesi son yedeğin tazeliği doğrulanır (00:18 timer + elle tetik).
5. Test anahtarları (`cf1339...`, `/tmp/*`) mainnet kararlarında kullanılmaz.
6. **Merkezi sunucu YOK (VPS dahil):** ağ, madencilerin birbirine ördüğü
   örümcek ağıdır. Hiçbir düğüm ayrıcalıklı değildir; tohum listesi dahil
   her adres değiştirilebilir listedir. Tek noktaya bağımlı tasarım
   reddedilir — tohum-0 (kurucu makinesi) dahil her düğüm düşebilir, ağ sürer.
7. **10-gün otonomi onayı (10 Eyl 2026, 1 hafta geçerli):** workspace + wiki +
   site dosyalarında okuma/yazma, servis restart, main'e push serbest.
   YASAK: disk silme, para harcama, dışarı mesaj atma. Push tokeni 1 hafta açık.
8. **Oturum hafızası (16 Eyl, süresiz):** her ilerleme bitiminde
   `00-OTURUM-DEVAM.md` §1 güncellenir; token bitiminde yeni profil
   §7 protokolüyle devam eder. Token'lar dosyaya yazılmaz.

## 1. Envanter (2026-09-06 doğrulandı)

### 1.1 Komuta (`NemesXSpace/komuta-rs`, 1776 satır)
- 17 endpoint: health, kayit, heartbeat, gorev, kanit, status, bakiye, arz,
  ledger, komut, denetim, denetim/sonuc, shard/ilan, shard, ara (POST+GET).
- Migration 001–007 uygulanmış. Canlı DB: ~1.4GB.
- Endpoint'ler: `:8787` HTTP + `:4003` P2P (nemes/shard abonesi).

### 1.2 Miner (`miner-core` + `miner-cli`)
- `miner-core`: mining.rs (466) + shard.rs (173) + llama.rs (277) + resources.rs.
- `miner-cli`: 435 satır — keygen, mine --simple (gerçek döngü), re-claim (30sn yoklama).
- Canlı: 2 miner (systemd), embed 1241/1242.

### 1.3 P2P (`nemes-p2p` workspace)
- GERÇEK: `nemes-core` (types/crypto/protocol/shard + MASTER_PUBKEY), `nemes-p2p`
  (TCP+Noise+Yamux+Gossipsub+mDNS+Identify), `nemes-cli` (keygen/node/imzala/komut-gonder).
- İSKELET (0–95 satır, içleri boş): nemes-consensus, nemes-embedding,
  nemes-node, nemes-protocol (0 satır!), nemes-storage, nemes-wallet.
- Karar bekleniyor: doldur veya workspace'ten çıkar (bkz. §3 Soru 3).

### 1.4 Model hattı
- `model/veri_kur.py` → `/srv/beyin/model-v01/` (train 1.1GB, 2.46M çift).
- `nemes-egitim/`: Qwen3-1.7B-Base + train.py (QLoRA) + cevapla.py + sorular.json.
- Tam eğitim sürüyor (adapter-v01, ~5900 adım). GPU %100.
- `model/hf-nemes-free-{1b,3b,8b,30b}/`: BOŞ (sadece README) — damıtma planı
  (`damitma-plani.md`, 70B Sovereign öğretmen varsayıyor) bu oturumun Qwen3-LoRA
  hattıyla BİRLEŞTİRİLMEDİ. Karar bekleniyor (bkz. §3 Soru 4).

### 1.5 Embed
- 6× LM Studio llama-server (1241–1246, `beyin_bekci.service`).
- Self-contained bundle `/home/d3str0y1ng/nemes-bundle` (194MB, CUDA 12.6, sm_61),
  `nemes-embed-bundle.service` ile :1251'de canlı. LM Studio ile kosinüs 1.000000.

### 1.6 Ops
- İzleme: `nemes-izleme-hizli.timer` (15dk) + `nemes-izleme-gunluk.timer` (06:00).
- Yedek: `komuta-backup.timer` (günlük 00:18), `/home/d3str0y1ng/nemes-testnet/yedek/`.
- Testnet dosyaları: `/home/d3str0y1ng/nemes-testnet/` (komuta.db, bin/, *.key, *.token).
- Paralel hatlar (bu oturumda dokunulmadı): Tauri `miner/` + `miner-api/` (557 satır),
  Python `/srv/beyin/kaynaklar/komuta_api.py`, `mini7-dongu.service` (/srv/beyin/testler).

## 2. Yapılacak değişiklikler (konsolide)

### Güvenlik (önce)
- [x] C1. Kökteki `nemes-x.2026-08-24.private-key.pem` kasaya taşındı
  (`/home/d3str0y1ng/.nemes-kasa/`, 700/600). TESPİT (içerik açılmadı):
  RSA-2048 anahtarı (Ed25519 DEĞİL → NEMES master imza anahtarı olamaz),
  Açık anahtar SHA256: `9e5fd207f83108b5f830353bed160b342a15456190f57c1660d9d358d60a1f99`,
  repo içinde referansı yok. Muhtemelen SSH/TLS artığı. Ne olduğu OPERATÖRE
  sorulacak (aşağıda Soru 5).
- [x] C2. Ölü dosya `nemes-testnet/+%F-%H%M).db` (0 bayt) silindi.

### Otonom depolama katmanı
- [ ] C4. Disk sınavı + katman ayrımı (`miner keygen --storage`, 100GB challenge).
- [ ] C5. Migration 008: `parcalar`, `parca_yerleri`, `miners.depolama_kota`.
- [ ] C6. Heartbeat'e disk raporu + ölüm ilanı (3 kaçırma).
- [ ] C7. `tip:"yedekle"` onarım döngüsü (kopya<5 ise transfer + ücret).
- [ ] C8. Yoklama endpoint'i (`POST /api/depolama/yoklama`) + düşürme kuralı.

### Temizlik (karar sonrası)
- [ ] C9. İskelet 6 crate kararı (Soru 3).
- [ ] C10. Tauri/Python hatları kararı (Soru 2) + emeklilik uygulaması.
- [ ] C11. Bu dosya her fazda güncellenir (bu madde dahil).

## 3. Açık karar soruları
1. **Tokenomik (KARAR VERİLDİ 08 Eyl — coin+halving):** Mevcut coin+halving
   kalır (`BATCH_ODUL_TABAN_MIKRO=2000`, `HALVING_BATCH=5M`). API-hakkı
   alternatifi elendi. Ledger/bakiye/arz endpoint'leri coin defteri olarak
   devam. (`docs/TOKENOMI.md` Faz 1 = pay birikimi, Faz 2 = H = gelir×%50 nakit.)
   Tokenomik KİLİTLİ (09 Eyl) + ETH-ölçeği (11 Eyl): tavan 210M, era1 0,0008
   NEMES/batch (800 mikro), `HALVING_BATCH` 50B. Enterprise
   fiyatları, Faz geçiş kriteri. Testnet `0.002` + 5M ile sürer; mainnet sabitleri
   (800 mikro + 50B) lansmanda tek commit'le geçirilir. Eski `odul-stratejisi.md` arşivde.
2. **Tauri `miner/` + `miner-api` + Python `komuta_api.py` (KARAR VERİLDİ 08 Eyl — dondur):**
   Tek hat Rust CLI (`nemes-miner`). `miner-api` workspace members'dan çıkarıldı
   (klasör duruyor, derlenmiyor); `komuta_api.py` → `komuta_api.py.donduruldu`;
   Tauri `miner/` zaten exclude'daydı, ellemedi. Hiçbiri canlıda değildi.
3. **İskelet crate'ler (KARAR VERİLDİ 08 Eyl — çıkar):** consensus, embedding,
   node, protocol (0 satır), storage, wallet → `nemes-p2p/_arsiv/` altına
   taşındı. Kod referansı yoktu (derlenmiyorlardı bile). İhtiyaçta geri alınır.
4. **Model hattı birleşmesi (KARAR VERİLDİ 08 Eyl — Qwen3-LoRA kilit):**
   `model/damitma-plani.md` → `damitma-plani.ERTELENDI.md`. adapter-v01
   (5900 adım, loss 1.48) ana hat; HF yükleme/benchmark buradan planlanacak.
5. **Kasaya taşınan .pem (KARAR VERİLDİ 09 Eyl — arşiv):** `nemes-x.2026-08-24.private-key.pem`
   (RSA-2048, Ed25519 değil → master olamaz) → `~/.nemes-kasa/arsiv/` (700).
   Muhtemelen SSL/TLS artığı. Gerekiyorsa operatör bulur.
6. **Sonraki korpus (GÜNCEL 09 Eyl):** wiki_tr/newscrawl/gut_en bitti + FAISS'li.
   cc100_tr embed'de (12 işçi, bitince merge+FAISS). wet_en atlandı (A kararı),
   paracrawl/caselaw/enwt eksik-kaynak. Komuta tek corpus sunuyor (GOREV_CORPUS).

## 4. Faz günlüğü
- 2026-09-18: **S9 VM topo düzeltildi (1 soket×4 çekirdek).** vcpuinfo kanıtı:
  8571/3/3/3sn → 29/26/25/39sn (4 çekirdek dengeli). XML yedeği
  `nemes-merge/paket/win10-test-onceki.xml`de. VM açık+DHCP tamam AMA madenci
  başlamadı (win-start.bat elle tıklama ister, otomatik başlama yok) →
  operatör 1 çift tık bekliyor. Dönünce hız ~3-4 kat beklenir.
- 2026-09-18: **nemes-p2p git'e alındı.** 5G dizinde sürümsüz kod riski
  vardı; ilk kayıt `217de57` (target/, llama.cpp-build, binaries hariç).
- 2026-09-18: **Bellek disiplini (B10).** Harita: qemu 8.3G + komuta ~8G
  (havuz) + llama 7x2.9G + opencode 1G. Tedbirler: drop_caches ile 9.6G
  nefes; 1243-1246 kapatıldı (+1.1G VRAM); sunucu_baslat.sh yalnız 1241/1242;
  havuz tavanı (HAVUZ_MAX_VEKTOR, default 1M, testli) — 1.001.203→1M kırpıldı.
  /api/ara 24 saatte 0 çağrı. Disk -30G faili meçhul; baz yazıldı.
- 2026-09-17: **Kabiliyet ilanı canlı (B23).** Migration 015 (`miner_yetenek`);
  nabızda `yetenek` (GPU/VRAM/roller, sinirli); `GET /api/filo` filo gorunumu.
  Canlı: 2 madenci "GTX 1080 Ti 11G embed,denetim,depolama,uretim" bildiriyor;
  win satirsiz (eski nabiz, uyumlu). Eslesme (gorev-sinifi yonlendirme) ilk
  metin-disi gorevde (B23b). Binary'ler `.20260917-yetenek.bak`'ta. Test 19/19.
- 2026-09-18: **Kira tam rollout (B20 devamı).** miner-b de kiraya geçti
  (drop-in); a+b kirada, win eski yolda. Dağıtım ~88→~76/5dk (kalan: win +
  denetim). Mesh-denetim b'de açık duruyor.
- 2026-09-17: **Gece nöbeti kuruldu (21:41) + kanarya emanet toplu tasfiye.**
  Kalan 365 kanarya emaneti de serbest bırakıldı (34.697 mikro, 3 madenci;
  kopya `/tmp/emanet-kanarya-tumu.sql`de, defter tozu sabit). Nöbet betiği
  güncellendi (kuyruk + komuta RSS/swap + OOM sayacı sütunları). Taban 21:41:
  win 417K kanıt (30.833'ü doğrulanmış-geçti, 0 kaldı), kuyruk 107K (eriyor),
  komuta 1.2G RSS + 5.6G swap (takipte), OOM sayacı=24 (sabah artmamalı).
- 2026-09-17: **Integration testleri (B3).** Router `app_router()`'a çıkarıldı;
  ephemeral-port iskelesiyle 3 HTTP turu: iskelet (health/404/auth), kira uçtan
  uca (kapanış+korunum), denetim turu (bayrak→görev→sonuç→doğrulama). Test
  18/18 (4 tur stabil). Yan ürün: `dagit_denetim` wiki'yi state havuzundan
  kullanıyor (cagri-basi baglanti curufesi bitti; ayni veri). Canlıda
  doğrulandı (20'lik denetimler akıyor). Binary `.20260917-b3.bak`'ta.
- 2026-09-17: **Teklif protokolü v1 canlı (B21).** Tasarım `docs/TEKLIF-PROTOKOLU-TASARIM.md`.
  Migration 014 (`teklifler`); `POST /api/teklif` (kapsama-reddi/stake-kilidi/
  cift-yonlendirme), `GET /api/bosluklar`, `GET /api/tekliflerim`; supurme
  onceligi (ATTACH'siz 3 sorgu); kapanista %1 bulucu payi + %80'de iade;
  vadede iade (yoklama dongusu). Miner `teklif` + `bosluklar` komutlari.
  Test 15/15 (tam yasam dongusu). Binary'ler `.20260917-teklif.bak`'ta.
- 2026-09-17: **Claim-kirası canlı (B20).** `POST /api/kira/al`: tek kilit +
  tek wiki okuma + toplu kayitlarla N alt-gorev (blake3 kor, tek duyuru).
  Miner `--kira` ile kuyruktan calisir, bitince tazeler; dususte eski yol.
  Olcum (unit-a kirada, b/win eski yolda): dagitim 1091->176/10dk, kilit
  seyreklesmesi, verim korunuyor (14.4 kanit/sn), 0 kira hatasi (2 hata eski
  restart araligindan). Binary'ler `.20260917-kira.bak`'ta. Test 14/14.
  NOT: unit-a=523d3c47, unit-b=a41b0d81 (tokenlar ilk gunden capraz; zararsiz,
  etiket karisikligi olmasin).
- 2026-09-17: **Gömme demeti diriltildi (B19, S4 önkoşulu).** Kök nedenler:
  (1) demet "self-contained" DEĞİLDİ — 3 .so LMStudio'ya symlinkliydi;
  (2) Eyl-4 stub + LMStudio 2.28.2 impl karışımı ABI segfault veriyordu
  (Eyl-10/11 ölüm sebebi bu); (3) start betiği `$1`'i iki kez geçiriyordu.
  Düzeltme: çalışan LMStudio 2.28.2 seti birebir içeri alındı (bin+8 .so+
  CUDA 11 runtime, ldd temiz) → 1.3G GERÇEKTEN bağımsız demet; :1259'da
  test edildi, 1241 ile kosinüs 1.000000. Test kapatıldı, :1251 dokunulmadı
  (yürürlükteki emir). S4'e hazır yapı (NVIDIA+CUDA gerekir). NOT: demet
  git dışı (nemes-p2p gibi) — paketleme push'la siteye verilecek.
- 2026-09-17: **WAN keşif v1 (B18).** libp2p `kad` + `/nemes/kad/1.0.0` + tohum
  adresleri + sabit kimlikler: komuta `p2p-komuta.key` (0600, PeerId
  `12D3KooWNgpfk…rd53K8` — tohum-0 tohumu), miner madenci-anahtarından türetir.
  `P2P_BOOTSTRAP` / `--bootstrap` bayrakları; bozuk adres yoksayılır.
  Test: mDNS-kapalı iki düğüm tohumla 3.5sn'de bağlandı. Canlı: komuta+miner-b
  yeni kimliklerle mesh'te, üretim+mesh-denetim sürüyor. Ara notlar: (1) D-Bus
  takılınca `sudo -n systemctl` kullan (polkit helper'ları asılı kalıyor).
  (2) `enable_mdns` bayrağı ÖLÜ bulundu (her zaman açıktı) → artık gerçek.
  (3) nemes-p2p/ git dışı — B15 notu geçerli.
- 2026-09-17: **Mesh denetim v1 canlı (B15).** Çift-yığın: komuta kapanan
  batch'te bayraklılar için deterministik atama duyurusu (`nemes/denetim`,
  blake3 korosu, `nemes-core::mesh_audit` paylaşımlı + testli) + metin kapısı
  (`GET /api/metin/:kor`, salt-okunur) + hakem (`denetim/sonuc` aynen).
  Eski `dagit_denetim` yolu duruyor (win .exe uyumluluğu). Miner `--denetim-mesh`
  (miner-b açık): duyuru→metin→embed→sonuç, 7dk'da 239 mesh denetimi, 0 hata.
  Kuyruk erimeye devam (127→125K). Binary'ler `.20260917-mesh.bak`'ta.
  Test: komuta 13/13 + miner-core 12/12 + nemes-core 10/10.
  Ara not: D-Bus bir kez takıldı (swap baskısı); miner-b 1dk durdu, döndü.
- 2026-09-17: **Yaşlı emanet serbest bırakıldı (50 satır, 4750 mikro).** Hepsi
  kanarya sentetiği (madde<0, 16 Eyl kesimi) — doğrulanamaz, sonsuz alarm
  üretiyordu. Operatör onayı (a) ile: coin+=miktar + ledger 'escrow', tek
  transaction. Kopya `/tmp/emanet-kopya-20260917.sql`'de. Defter tozu sabit
  (195/200/0) — kitaplar denk.
- 2026-09-17: **5-dk tetikleyici çözüldü (fail biliniyordu, kaynak meçhuldü).**
  Tuzak (PPID izi) yakaladı: tetikleyici harici döngü/cron DEĞİL —
  `beyin_sira.service.d/after.conf` içindeki `Wants=beyin-sunucu.service`
  (tasarım: tur öncesi embed sunucuları ayakta olsun). KillMode düzeltmesiyle
  bu tasarım ilk kez gerçekten çalışıyor (1243-1246 doldu, 6/6 canlı).
  Tuzak satırları geri alındı, iz logu duruyor.
- 2026-09-17: **Toplu-kanıt girişi (B14).** `POST /api/kanit/toplu` (EK yol, cap 100):
  gorev basina 20 HTTP yerine 1. Her kalem mevcut `kanit()` kodunu cagirir
  (odul/emanet/kapanis birebir; kalem hatasi batch'i durdurmaz). Tekil yol aynen
  duruyor (win .exe uyumlulugu). Miner toplu gonderir, 404'te tekliye duser.
  Test: 12/12 + 12/12 (yeni `test_toplu_yasam_dongusu`: kapanis + odul korunumu +
  cift-kayit reddi + bos/asiri reddi). Binary'ler `.20260917-toplu.bak`'ta,
  komuta+miner-a/b deploy edildi, uretim saglikli (425 batch/5dk, 0 hata).
- 2026-09-17: **Shard-claim disiplini (B13).** Tablo çürümüştü: 9 "aktif" ilanın
  8'i bayattı, hepsi donuk tr cursor'una (4724762) yığılmıştı; miner `--corpus tr`
  bayrağı mesh ilanlarını yanlış etiketliyordu (komuta newscrawl sunarken).
  Düzeltme: a/b unitlerinde `--corpus newscrawl_tr` (drop-in `z-*`, token.conf
  sonrası; yalnızca ilan etiketini etkiler). Sonuç: ilanlar canlı cursor'a
  (1.72M) çıpalandı, aktif→tukendi döngüsü işliyor. Ölçüm: gorev ~2.3/sn
  (0.76/miner/sn; 30K projeksiyonu ~23K/sn — federasyon eşiği), kilit ort 0ms
  (metrik 16 Eyl'den canlı, 500ms warn eşiği). Bayat satırlar silinmedi (ölçüsüz,
  okuma-anında expiry yeterli).
- 2026-09-17: **İlk canlı mesh (B12 provası, miner-b).** `--p2p-dinle` tek miner'da
  açıldı (drop-in `z-p2p-prova.conf`, token.conf sonrası; a+win poll devam).
  Kanıt: mDNS discovery çift yönlü + 29 `connected` + gossipsub `nemes/gorev`
  aboneliği (kod) + dinleyici `◈ gorev duyurusu dinleniyor`. Üretim kesintisiz
  (0 hata). Dersler: (1) ShardRelay+P2PNode aynı `--p2p-port`u paylaşamaz
  (sabit port çakışır, hata iletisi boş — küçük log ayıbı); `0` (rastgele) kullan.
  (2) drop-in sırası: token.conf'u ezmek için `z-` öneki. (3) idle→uyanma yolu
  gözlenemedi (görev bolken miner hiç boş kalmıyor) — mekanizma kod-doğrulamalı,
  ilk kıtlıkta journal'dan teyit edilecek. Dinleyici AÇIK bırakıldı (mesh tohumu).
- 2026-09-17: **Denetim kuyruk zehirlenmesi çözüldü (B5).** 16 Eyl 20:33'ten beri
  17 saat denetim dağıtımı YOKTU (kuyruk 175K+). Kök neden: kanarya sentetikleri
  (madde_id<0, Güvenlik Md.3) kuyruk başına dizilmişti; `dagit_denetim` wiki'de
  ozet bulamayınca refs boş dönüp sessizce None veriyordu (zehirli-kuyruk-başı).
  Düzeltme: secimlere `madde_id>=0` + kanıt girişinde sentetiğe spot bayrağı yok
  (ödeme akışı aynı). Sonuç: 20'lik tam denetim görevleri + Hepsi `gecti`
  (cos≥0.997), kuyruk -15.7K/sa eriyor (~11 saatte sıfırlanır). Binary
  `.20260917-denetim.bak`'ta. Test 11/11.
- 2026-09-16: **Ölçek-1 deploy edildi.** Kilit metriği (500ms warn + 1000'de
  ortalama) + WAL/NORMAL/30sn-timeout + stres tablosu (`docs/TOKENOMIK-STRES.md`).
  Canlıda -wal/-shm doğrulandı, batch'ler kapanıyor. Komuta DB 2.6G (üretim
  büyümesi normal). Test 11/11.
- 2026-09-16: **Gece nöbeti (win madenci).** 30dk örneklemeli `nemes-merge/gece-gozlem.sh`
  (18 tur ≈ 9 saat): win pay/kanıt/spot/geçti/kaldı + toplam + escrow + llama +
  servis + hata + disk. Taban 23:19: win 6971 pay, ağ 1.55M kanıt, 49K emanet.
- 2026-09-16: **İLK WINDOWS KANITI ✅ (win10-test).** Zincir: görev alındı ama
  embed 10060 zaman aşımına düştü → sebep UFW INPUT (1247 kapalı) → kural açıldı
  (192.168.122.0/24 → 1247/8080) → retry döngüsü kendiliğinden toparladı, ilk
  kanıtlar aktı (352 kanıt/pay 351). BUG-2 retry tasarımının ilk canlı kanıtı.
- 2026-09-16: **VM sanal kablo kapalıymış (`link state='down'`).** Windows "hiçbir
  ağ yok" diyordu, sayaçlar çalışıyordu (DHCP önceden alınmış). `domif-setlink
  up` ile açıldı. Ders: `domiflist` yetmez, `dumpxml` link satırı da okunur.
- 2026-09-16: **VM internet kesintisi (UFW/Libvirt çakışması).** `LIBVIRT_FWO`
  zincirinin başına yabancı REJECT satırı girmişti (VM→dış tüm trafik ölüyordu,
  DHCP/host-yönü çalıştığı için gizli kaldı). Kural silindi, NAT doğrulandı.
  Tekrarlarsa: `iptables -D LIBVIRT_FWO 1`. Kalıcı çözüm: UFW reload sonrası
  kontrol (§12.3'e eklenecek).
- 2026-09-16: **VM dosya köprüsü v3 (USB CD-ROM).** v2 ham ISO'yu disk diye
  takmıştı, Windows bölüm tablosu aradı (D: okunamadı). CD-ROM+USB olarak
  tekrar takıldı (ISO9660 optik sürücü). Ders kaydı v2 maddesinde.
- 2026-09-16: **VM dosya köprüsü v2 (USB-disk).** İlk deneme virtio veriyoluna
  takıldı (Windows sürücüsüz görmedi), SATA hotplug chipset'te yok → ISO USB
  disk olarak canlı takıldı. Ders: KVM'ye takılan her aygıtın Windows sürücüsü
  doğrulanır (virtio-drop, SATA-reboot, USB-canlı).
- 2026-09-16: **Windows ilk çalıştırma hatası + düzeltme.** `win-start.bat`
  Downloads'a gidiyordu, dosyalar USB/Masaüstündeydi → `%~dp0` (bat'ın kendi
  dizini) düzeltmesi. Dağıtım: mini-CD (sanal cdrom, düzeltilmiş bat + spice
  installer). Ders: bat'lar konum-bağımsız yazılır.
- 2026-09-16: **Windows VM dosya köprüsü (USB passthrough).** Flash'a NEMES/
  klasörü (exe+bat+token+kurulum+spice-guest-tools) yazılıp cihaza canlı takıldı
  (hotplug). Sıradaki: spice kurulumu (pano açılır) + win-start.bat ilk kanıtı.
- 2026-09-16: **Windows VM kuruldu (win10-test).** Flash'taki 2019 Win10
  kurulumundan xorriso ile 6.6G önyüklenebilir ISO üretildi, KVM: 8G/4CPU/60G
  qcow2, SATA+e1000 (virtio sürücüsüz kurulum), NAT. `nemes-miner.exe` (28M)
  çapraz derlendi, VM'ye aktarım + dış kayıt provası operatör kurulumunu bekliyor.
- 2026-09-16: **Tor ucu provası GEÇTİ (tohum-0 0.3'e sayılır).** VM clipboard
  olmadığı için host-Tor kullanıldı (check.torproject IsTor:true). Tor çıkışı →
  dış kayıt 200 + status 200 (WAF-skip Tor'u da kapsıyor). Prova satırı silindi,
  tor servisi durduruldu. Kalan: gerçek dış makine + 4003 modem.
- 2026-09-16: **Alarm triyajı (izleme çalışıyor, 3 alarm).** (1) Defter tutarsızlığı
  YANLIŞ alarm: eşzamanlı ödemede iki ayrı sorgu yarışıyordu → TEK sorguluk
  atomik kontrole çevrildi, sustu. (2) Çift-dağıtım patlaması: corpus-kör sayaç
  eski `tr` + yeni `newscrawl` ID çakışmasını sayıyormuş (örnek: madde 62074 iki
  corpus'ta iki belge) + kanarya negatifleri → corpus-bazlı sayaca çevrildi,
  sustu; aynı-görev çifti UNIQUE ile zaten imkansız. (3) Denetim kuyruğu GERÇEK
  birikim (5000+): bayrak girişi denetim çıkışının ~50 katı → DENETIM_BATCH
  5→20 büyütüldü, deploy edildi (`.20260916-denetimbak.bak`). İzleme 1 alarma indi.
- 2026-09-16: **CF WAF-skip CANLI (operatör kurdu, doğrulandı).** Çıplak UA
  (`WAF-Probe/1.0`) ile dış kayıt 200 döndü; prova satırı silindi. Dış miner
  kapısı resmen açık → tohum-0 0.3 kapanışa bir adım (kalan: gerçek dış makine).
- 2026-09-16: **Operatör kiti v1 taslağı** (`docs/`, onay bekliyor) + bitiş
  analizi (`docs/BITIS-ANALIZI.md`: teknik %70+, darboğaz operatör/hukuk/ekonomi).
- 2026-09-16: **Corpus kesimi newscrawl_tr (56.5M madde).** caselaw 454/454
  bitince drop-in güncellendi + restart; miner'lar yeni görevlerde üretiyor
  (tamamlanan 100+, vectors/s>0, canlı DB'de `newscrawl_tr:*` kapanışlar).
- 2026-09-16: **Dalga A CANLI: caselaw kesimi.** 1242 kaldırıldı (2/2 llama ok);
  drop-in `corpus.conf` (GOREV_CORPUS=caselaw) + restart; miner'lar üretiyor
  (caselaw 20/20 kapanışlar + spot bayrakları canlı DB'de). İlk drop-in denemesi
  dizin yokluğundan boşa düştü, yakalandı + düzeltildi (ders: mkdir önce).
  Site paketi `nemes-xspace.github.io/miner/` altında HAZIR (zip+SHA256+KURULUM,
  commit/push ONAYI bekliyor).
- 2026-09-16: **Dalga A başladı: caselaw hattı + binary paketi.** wiki_caselaw
  454 madde embed+merge (0.2dk) + FAISS nlist32 (883K, 454 nokta 512 kümeye
  sığmadı). Kopya DB'de dağıtım provası: kayıt→20'lik görev→kanıt kabul.
  Canlıya dokunulmadı. Paket `nemes-merge/paket/nemes-miner-v0.2.0-linux.zip`
  (5.4M, SHA256'lı, --help doğrulandı) — siteye yükleme ONAYI bekliyor.
  SIRADAKİ: canlı corpus kesimi (GOREV_CORPUS) onayı.
- 2026-09-16: **Paranoyak tur-1 (Garanti-9 kuruldu).** Bulgular: izleme servisi
  ölü bulundu (bayat beklentiler) → düzeltildi (0 alarm); miner token'ları
  ps'te çıplak bulundu → env-file'a alındı (drop-in, 0 eşleşme doğrulandı);
  yedek restore provası geçti (integrity ok, 19 tablo); repo leak taraması temiz
  (vite FP hariç); cert 9 Ara (84 gün); / %95 baskısı → cargo clean 12.8G +
  yedek rotasyonu 9 dosya + soğuk arşiv /srv/beyin'e taşınıyor.
  Hafızaya §12 protokol + §11 kuralları işlendi.
- 2026-09-16: **Tohum-0 gerçeği + CF bot bulgusu.** Tünel config okundu:
  HTTP(S) Cloudflare Tunnel ile CANLI (modem gereksiz), P2P 4003 tünelden
  geçmez (mesh için modem yönlendirme ayrıca gerekir). Dış prova: curl 200,
  kayıtsız UA 403 (1010) / tarayıcı UA 200. Çözüm iki kol: miner UA
  (`NEMES-Miner/0.2`, deploy edildi, dış status 200 doğrulandı) +
  operatör CF WAF-skip kuralı (/api/*). Test satırları canlı DB'den silindi.
  TOHUM-0-KURULUM.md gerçekle uyumlu hale getirildi.
- 2026-09-16: **Kapsam kilidi: SADECE MADENCİLİK (operatör kararı).** Model
  çıkarma/damıtma/HF işleri ertelendi (v1.2'de 1.4 + 3.2 ERTELENDİ işlendi).
  Site Y2 iddiası yayınlanmayacak (kaldır/erte). `model/` dizini donduruldu.
- 2026-09-16: **Kalanlar turu: v1.2 + 4 taslak.** Yol haritası v1.2 ayrı dosyada
  (`Belgeler/`, v1.1'e dokunulmadı): 1.1 sabitleri 800 mikro/50B, DONE işaretleri,
  metrik 110M+, %10 sertleştirme kotası, vesting şartı, bütçe yaşar-sırası.
  Yeni taslaklar (`docs/`, onay bekliyor): KURUCU-ANAHTAR (K1→K3), GDPR-SILME
  (tombstone+rebuild+lisans+PII), TOHUM-0-KURULUM (DNS+modem checklist),
  SITE-YAMA-LISTESI (Y1-Y5, site reposuna uygulanacak).
- 2026-09-16: **E2E escrow provası GEÇTİ (kopya DB, :18787).** 20 kanıt→batch
  kapandı (1400 temiz + 600 emanet = 2000 tam); geçiş: cos 1.0→emanet serbest
  + ledger 'escrow'; kalma: 2 ret (0.73/0.74)→slash 2000 + strike + emanet yandı;
  legacy skor 400; salt 32B. Canlıya dokunulmadı (escrow 0, cursor 4724762),
  kopya silindi. Kanıt logları: `nemes-merge/e2e/`.
- 2026-09-16: **Shard 12×2.2G arşivlendi** (26G → `nemes-merge/arsiv/`,
  spot: shard_0 9.106.643 madde okundu). /srv/beyin 242G boş. Silinen yok.
- 2026-09-16: **Sıra zamanlayıcı temizliği.** `beyin_sira.service` sonsuz
  `Restart=always` (20sn) döngüsünden oneshot+timer'a çevrildi
  (`beyin_sira.timer`: 5dk, `temizlik.conf` drop-in). Çakışma `flock -n`
  ile engelli (kilit: `kaynaklar/.beyin_sira.lock`; /run kullanıcıya kapalıydı).
  Çift satır log bitti (stdout journal'a, dosyaya yalnız script yazar).
  Doğrulama: 13:51 timer tetiklemesi başarılı, sonraki 13:56 kurulu.
- 2026-09-16: **Komuta localhost bind (d247ab8).** HTTP `:8787` artik yalnizca
  127.0.0.1'de (BIND_ADDR env, default localhost; P2P 4003 etkilenmez). Dogrulama:
  ss'te 127.0.0.1:8787, miner'lar kopmadan polling, Caddy TLS yolu 200.
  Eski binary `.20260916-bind.bak`'ta. DB degisikligi yok (yedek gerekmedi).
- 2026-09-16: **Denetim-2 deploy edildi (618d628).** B-4 migration 013 `spot_salt`
  (günlük salt) + STRICT_DENETIM env (default 0) + BUG-2 retry + B-5 hash pini +
  parse OR-IGNORE resume (wiki_parse_par.py, git-dışı). Yedek:
  `yedek/komuta-2026-09-16-manuel-618d628.db`; eski binary'ler
  `.20260916-denetim2.bak`'ta. Canlıda `escrow`+`spot_salt` doğrulandı,
  3/3 servis active, miner polling. Test: 12/12.
- 2026-09-16: **Denetim B-1/BUG-1/B-3 deploy edildi (144def0).** SPOT %1→%10;
  batch dagitimi emanetli (migration 012 `escrow`, CREATE-only); geciste serbest
  bırakma, slash'ta yakma; legacy istemci skoru kaldirildi; miner embed_batch
  index-yerlestirmeli. Yedek: `yedek/komuta-2026-09-16-manuel-144def0.db` (1.5G);
  eski binary'ler `.20260916-escrow.bak`'ta. Restart: komuta PID 87372 (:8787+
  :4003 canlı, health ok), miner'lar polling (corpus tukenik, 204-bekliyor).
  E2E escrow ilk gercek dagitimda kanitlanacak (su an dagitilacak gorev yok).
- 2026-09-16: **cc100 arşiv taşındı (silme yok).** Merge kapsaması 1800 örnekle
  doğrulandı (12 shard × 150 PK-seek, eksik=0 bozuk=0). emb 12x7G (84G) + orig-bak
  25G → `/home/d3str0y1ng/nemes-merge/arsiv/` (mv, dosya dosya doğrulamalı).
  /srv/beyin 104G→215G boş. Arşivden emb_0 spot-okundu (9.105.979). Canlı:
  symlink merged (118G) + FAISS-24 + orig yok (arsivde).
- 2026-09-15/16: **cc100_tr KAPANIŞ — embed+merge+FAISS DONE.** 12 işçi 15 Eyl
  0 yeniyle kapattı (w9 son 34.130). Toplam **109.271.443 vektör + 7.635 atlanan**
  (%101.2). Merge `/` bölmesine kopya üstüne (25G→118G, 1.3sa, INSERT OR IGNORE,
  GENEL TOPLAM birebir), canlı DB'ye yazılmadı; swap symlink ile (eski 25G
  `.orig-bak`'ta, silinen yok). FAISS-24 66dk (3942s): 4.1G faiss + 834M ids,
  ntotal=109271443 doğrulandı. Disk: /srv/beyin ~104G, / ~193G. Kalan: emb 12x7G
  arşiv kararı + DURUM commit. 16 Eyl reboot (llama 0/6, görev yok).
- 2026-09-11: **Embed yavaşlama teşhisi + çare** — llama.cpp 2.28.2 ~2 saatte 4x
  (ölçüm 1241: 3435→764/dk; restart 4944/dk'a döndürüyor, tam cure). RSS sabit
  (leak yok), hata yok, doküman/token boyu sabit → slot zamanlama patolojisi.
  Çare: `embed_bekci.sh`'e dönüşümlü gençleştirme (sadece sağlıklıken, 90dk'dan
  yaşlı tek sunucu, 30dk arayla) + `KillMode=process` drop-in (servis restart'i
  cgroup katliamı yapmasın — 15:59'daki restart 6 sunucuyu SIGTERM'lemişti).
  Üretimde kanıtlandı (taze 6960/dk vs bayat 680/dk, 10x). 20:36'dan itibaren
  kapı 30dk→15dk (rotasyon 90dk, ETA ~1,5 gün). 12 Eyl 00:08: kapı 15dk→5dk
  (rotasyon 30dk, çürüme ilk 30dk'da olduğu için); ateşleme doğrulandı
  (1243, 5423sn, AYAKTA). Kapsam revizyonu: 108M madde, kalan ~102M → ETA ~4-5 gün.
  DÜZELTME (12 Eyl 18:11 kontrolü): 5dk kapı rotasyonu kısaltmamış — yaş kapısı
  (5400sn) bağlıyordu, merdiven 15dk'da kalmıştı. Gerçek düğme yaş kapısı:
  5400→1800sn indirildi, rotasyon gerçekten ~30dk'ya indi (ateşler 5-6dk arayla,
  yaşlar 4026sn'ye converging). ETA ~3,5-4 gün.
- 2026-09-12 02:49: **GPU tavan 250→300W + persistence mode** — gerekçe: taze
  sunucularda GPU %97 pinleniyordu (250W binding). Sonuç: ısı sabit (52°C),
  Xid/hata sıfır, Tctl 56°C. 375W'a çıkılmadı (VRM riski, azalan getiri —
  Pascal verimlilik cliff'i). Not: hız A/B'si rolling çürümesiyle karışık
  olduğu için izole edilemedi; zarar yok, tavan rahatladı.
- 2026-09-12 03:05: **GPU tavan 300→250W GERİ ALINDI (operatör emri)** — kart elleme
  yok; kullanım iş yüküyle yükseltilecek. Gözlem: kullanım %36-97 salınıyor
  (ort ~%50-60); darboğaz kart değil sunucu hattı.
- 2026-09-11: **Tavan kota ilkesi kilitlendi (garanti çekirdeği)** — docs/TOKENOMI.md
  §2b: günlük dağıtım ≤ HALVING_BATCH/1460 (era1 ~34,2M batch/gün), kota dolunca
  dağıtım durur + ertesi güne devreder, miner-başı tavan (sybil freni). En hızlı
  senaryoda bile era ≥4 yıl → anlamlı mining ≥32 yıl. Kod Faz B'de (tetik: %80 ×
  7 gün); erken dönemde fren yok.
- 2026-09-09: **Güvenlik mimarisi kodlandı (deploy bekliyor)** — docs/GUVENLIK-MIMARISI.md +
  migration 011 (kor_esleme/kanaryalar/kanarya_dagitim/kara_liste, kopya DB'de
  test edildi) + kör ID (dagitim basina rastgele kor, kanit/denetim donusunde
  cozum) + corpus hash kodu + kanarya ekici (%4, negatif sentetik ID) +
  `/api/kanarya/kontrol` + komut `imha` (ban+pay sifir) / `affet` genisletme +
  kara liste kapilari (gorev+kanit) + log hijyeni (metin yok, dogrulandi).
- 2026-09-09: **Güvenlik deploy edildi** — komuta restart (PID 1551458, hata yok),
  migration 011 canlıda (4 tablo + 5 kanarya), `/api/kanarya/kontrol` provası
  yeşil (`eslesti:true`), miner'lar bağlı. Eski binary `.20260909-guvenlik.bak`'ta.
- 2026-09-08: **S2/S3/S4 kapatıldı** — miner-api members'dan çıkarıldı,
  komuta_api.py donduruldu, 6 iskelet crate `_arsiv/`'de, damıtma planı
  ertelendi (Qwen3-LoRA kilit). `cargo check --workspace` temiz. Hepsi geri
  alınabilir (silme yok, taşıma var).
- 2026-09-08: **R4/P2P görev dağıtımı kodlandı (deploy bekliyor)** — `nemes/gorev`
  GossipSub: lib abone listesine eklendi + `GOREV_TOPIC`; komuta `gorev_kaydet`
  sonrası hafif duyuru (`gorev_id`+corpus, hassas yük yok) kuyruğa atar,
  shard-abone görevi mesh'e yayınlar (`P2P_GOREV_YAYIN=0` ile kapatılabilir);
  miner `--p2p-dinle` ile dinleyip 5/10sn poll yerine anında uyanır
  (default kapalı, canlı miner davranışı değişmez). `cargo check` + release
  build yeşil (`target/release/komuta-rs`, `nemes-miner`). Canlıya deploy
  (komuta+miner restart) ONAY bekliyor. Sıradaki: deploy +   mesh provası.
- 2026-09-08: **R4/P2P deploy edildi** — eski binary'ler `.20260908.bak`'ta,
  servisler restart (komuta PID 1098778, :8787+:4003 canlı, hata yok).
  Binary kanıtı: `strings`te `nemes/gorev`, miner `--p2p-dinle` flag canlı.
  Canlı mesh provası kısmi: corpus tr tükenmiş (HTTP 204, miner'lar
  `bekleniyor`), dağıtım olmayınca duyuru da ateşlenmedi — ilk gerçek
  dağıtımda otomatik ateşlenecek. Miner'larda `--p2p-dinle` bilerek kapalı
  (yeni corpus gelince açılacak).
- 2026-09-07: **CPU guc orani:** tum policy'lerde `scaling_max_freq=2960000` (2.96GHz
  = donanim maksimumunun ~%60'i, GPU ile ayni oran). NOT: tek sayili cekirdekler
  SMT kardesidir, zamanlayici bilerek bos tutar (isi tasarrufu) — zorla yayma
  YAPILMADI (isiyi artirirdi). Reboot sonrasi governor powersave'a donerse
  `cpupower frequency-set -g powersave` + yukaridaki max deger tekrar uygulanir.
- 2026-09-07: **GPU güç tavanı:** 250W → 200W → **150W** (`nvidia-smi -pl`), persistence
  mode açık. NOT: reboot sonrası tavan sıfırlanırsa `sudo nvidia-smi -i 0 -pl 150`
  tekrar uygulanır. Eğitim ~%25-35 yavaşlar, hesap buna göre yapılır.
- 2026-09-07: **C8 tamamlandı** — yoklama döngüsü canlı: migration 010
  (`yoklamalar`), 5dk arka plan görevi (süre-doldu kapatma + dusurme kontrolu +
  tur limiti 3), `GET/POST /api/depolama/yoklama[/sonuc]` + hedefli
  `/uret` endpoint'i, miner 60sn tick'te otomatik cevaplar
  (`parca_hash_aralik`, 9 test yeşil). Prova: gecti→kilit, 2×fail→kota=0
  (dusuruldu:true), sunucu-hesaplı kosinüs eşiği. Canlıda 30 parça × 2 kopya
  artık otonom denetimde. Sıradaki: Soru 6 (sonraki korpus) + R1 tokenomik kararı.
- 2026-09-07: **C7 tamamlandı** — onarım döngüsü canlı: tohum (yedek→64MB parça,
  yedek-dizini hapsi), `GET /api/parca/indir`, `tip:"yedekle"` dağıtımı (kota>0,
  kopya<3), heartbeat kapanış + 100 mikro ücret, ölüm eşiği 300sn. Canlı kanıt:
  46 parça tohumlandı, 60 onarım kapandı, 2'şer kopya, 6000 mikro ücret, A==B
  bayt-bayt aynı. Miner `--depolama` + unit'lere eklendi, binary'ler deploy edildi.
- 2026-09-06: **C6 tamamlandı** — heartbeat disk raporu iki taraflı: komuta
  `HeartbeatReq{parcalar,depolama_kota}` kabul eder (upsert + kota günceller,
  bozuk hash atlar, bodsuz gövde geriye uyumlu); miner `mine --depolama` ile
  60sn'de taahhut+kota+parça listesi bildirir. Prova (kopya DB): 2 parça kaydı,
  kota 100GiB, miner logunda "1 parca bildirildi". Sıradaki: C7 onarım döngüsü.
- 2026-09-06: Kayıt dosyası oluşturuldu. C1+C2 uygulandı. Yedek teyit edildi (00:18, 1.5GB).
- 2026-09-06: **C4 tamamlandı** — `miner-core/src/depolama.rs` (baraj 100GiB, blake3
  sınavı yaz+oku+dogrula+sil, taahhut JSON) + `miner keygen --storage --yol --boyut-mb`.
  9/9 miner-core testi geçiyor. Canlı prova: 333GB diskte 64MB sınav geçti, taahhut
  yazıldı, sınav artığı silindi.
- 2026-09-06: **C5 tamamlandı** — migration 008 (`parcalar`, `parca_yerleri`,
  `miners.depolama_kota`). Yol üstünde checksum onarımı gerekti (001–007 dosyaları
  commit sonrası EOF düzeltmesi yemiş, sqlx reddetti): kopya DB'de prova edildi,
  taze yedek (14:18) alındı, canlıya uygulandı. Veri kaybı yok.
- 2026-09-06: **MİLATTIR — wiki_tr %100 kapsama.** 975.570 uygun maddenin tamamı
  kanıtlandı (976.461 kanıt, 891 tarihsel çift). Cursor wiki sonuna dayandı.
- 2026-09-06: **Süpürme tıkanıklığı düzeltildi.** Sınırsız tarama penceresi dağıtımı
  kilitliyordu (~38 saattir kimse görev alamıyordu). Pencere 5000 id ile
  sınırlandı. Kalan yavaş sorgular = vektör havuzu tazeleme (2 dk'da bir, arka plan).
- 2026-09-06: **Miner 204 hatası düzeltildi.** `is_success()` 204'ü geçirip boş
  gövdeyi JSON parse ediyordu (sonsuz EOF hatası). Artık temiz bekleme.
  Sıradaki: C7 (onarım döngüsü) + SONRAKİ KORPUS kararı (Soru 6).
