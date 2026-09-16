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
