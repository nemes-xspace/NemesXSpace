# NEMES-X — OTURUM DEVAM DOSYASI (tek hafıza)

> **Amaç:** Token bitince / yeni profil açılınca kaldığımız yerden devam.
> Yeni oturumda İLK bunu oku, sonra §7'deki sırayı izle.
> **Kural:** Her ilerleme bitiminde bu dosyanın §1'i güncellenir
> (ALTYAPI-KAYIT.md Garanti-8). Tarihsiz satır bırakılmaz.

---

## 1. SON SNAPSHOT — 16 Eyl 2026 13:35 (+03, date ile doğrulandı)

| Kalem | Durum |
|---|---|
| Git | Son push: NemesXSpace `2318ce5` ✅ (6 commit) + site `a90ddf1` ✅ | Bekleyen: `git status -sb` çıktısındaki `[önünde: N]`
  sayısı esastır (bu satırdaki sayı YAZILMAZ — commitlendikçe eskir;
  N'i `git log origin/main..HEAD --oneline` ile gör) |
| Denetim-2 | B-4 (salt) + STRICT_DENETIM + BUG-2 + B-5 + parse-resume **canlıda**
  (`618d628`, migration 013 uygulandı, 3/3 servis active) |
| Testler | miner-core 12/12 + komuta-rs 10/10 (`48986cc`; batch/slash yaşam döngüsü gerçek handler'la) |
| İzleme | 3 alarm triyajlandı: defter (yanlış alarm, düzeltildi) + sayaç (corpus-kör, düzeltildi) + kuyruk (gerçek, batch 20'ye büyütüldü `9f35d1a`) |
| Güvenlik | Komuta HTTP localhost-only (`d247ab8`, BIND_ADDR); P2P 4003 açık |
| Tören | 1024-karakter parola doğrulandı (16 Eyl): diskte kopyası YOK (kartta),
  history/repo sızıntısı YOK; redis: `offline-ceremony/` 3 script |
| Sıra | Timer kalıbı (`beyin_sira.timer` 5dk + flock; `e5cfdb9`) |
| Tohum-0 | HTTP(S) tunnel ile CANLI ✅; eksik: CF WAF-skip (/api/*, sende) + 4003 modem (mesh için) |
| Yol haritası | v1.2 ayrı dosyada (`Belgeler/`, 1.1 sabitleri düzeltildi) |
| Taslaklar | Kurucu-anahtar + GDPR + Tohum-0 + Site-yama (`docs/`, onay bekliyor) |
| Dalga A | ✅ CANLI: caselaw 454/454 bitti → **newscrawl_tr kesildi** (üretim:
  26K+ kanıt, 1665 emanet satırı/166K mikro CANLIDA — mekanizma üretimde doğrulandı) |
| cc100_tr | KAPANDI ✅ 109.271.443 vektör, merge 118G, FAISS-24 (4.1G+834M, ntotal doğrulandı) |
| Denetim düzeltmeleri | B-1 (escrow+%10) + BUG-1 + B-3 **canlıda** (migration 012 uygulandı, restart yapıldı) |
| Hafıza sistemi | Bu dosya + `durum-anlik.sh` + Garanti-8 ✅; anatomi §10 eklendi, tatbikat yapıldı |
| Arşiv | emb 12x7G + bak 25G + shard 12x2.2G → `/srv/beyin/arsiv_cc100/` (25 dosya, 161G, silinen yok) |
| Servisler | 6/6 active + beyin_sira (ikisi de betikte sorgulanır) |
| Llama | 1241 tekil `ok` (sorgu kapısı); 1242-1246 kapalı (görev yok) |
| Miner'lar | Sağlıklı polling, `gorev:bekleniyor` (corpus tr tükenik — bilinen) |
| Disk | /srv/beyin **104G** boş (%89; arşiv buraya taşındı), / **210G** boş (%77) |
| FAISS | 24 index, toplam ~7.4G (cc100 ~4.9G faiss+ids, diğer 23 ~2.5G) |
| Yedek | komuta-backup timer günlük çalışıyor (saat ~00:1x bandı); son manuel:
  `yedek/komuta-2026-09-16-manuel-618d628.db` (1.5G) |
| E2E escrow | ✅ Kopya DB'de kanıtlandı (16 Eyl): geçiş+kalma+legacy+salt |
| Arşiv | emb 12x7G + bak 25G + shard 12x2.2G → `/srv/beyin/arsiv_cc100/` (25 dosya, 161G, silinen yok, spot doğrulandı) |

## 2. KRİTİK YOL (07 Eyl revize)

1. ULTRA ✅ → 2. newscrawl+gut ✅ → 3. eğitim adapter-v01 ✅ →
4. **cc100 ✅ (15 Eyl: embed+merge+FAISS-24)** → 5. FAISS ✅ (24 index, 7.4G) →
6. Miner v0.2 Windows (iskelet, imza+taahhüt operatörde) →
7. Komuta+Ödül (B-1/BUG-1/B-3 canlı; B-4/B-5/B-6 + imza Faz-2 bekliyor) →
8. Debian port (başlanmadı).
Blokeliler: wet_en (disk), paracrawl (S3 404), caselaw (hesap) — hepsi `eksik-kaynak`.

## 3. CANLI SİSTEM HARİTASI

```
Portlar: 1241 (llama, ACIK) | 1242-1246 (kapali) | 8787 komuta HTTP |
         4003 komuta P2P | :1251 bundle KAPALI (dokunma)
Servisler: beyin_bekci, nemes-komuta, nemes-miner-a/b, caddy, cloudflared,
           beyin_sira, komuta-backup.timer (00:18), nemes-izleme timer
Anahtar yollar:
  Ana DB (SYMLINK!): /srv/beyin/wiki/wiki_cc100_tr.db
      -> /home/d3str0y1ng/nemes-merge/wiki_cc100_tr_merged.db (118G)
  FAISS-24: /srv/beyin/wiki/wiki_cc100_tr.faiss (4.1G) + .ids.npy (834M)
  Arsiv: /srv/beyin/arsiv_cc100/ (25 dosya, 161G)
  Komuta DB: /home/d3str0y1ng/nemes-testnet/komuta.db (1.5G)
  Komuta bin: /home/d3str0y1ng/nemes-testnet/bin/ (.bak'lar yanında)
  Test loglari: /home/d3str0y1ng/nemes-merge/{merge,verify,arsiv,faiss_cc100}.log
```

## 4. SON COMMITLER (yeniler üstte; tam liste `git log` ile)

```
a02d493 hafiza (bu dosya+betik+garanti-8, PUSH BEKLIYOR) |
43ee3ec deploy kaydi (PUSH BEKLIYOR) |
144def0 B-1/BUG-1/B-3 kod (PUSH BEKLIYOR) |
b285836 arsiv (PUSH'LU) | c71fb4b cc100 kapanis (PUSH'LU) |
03e07c6 yas kapisi 1800sn | c3a91be yol haritasi 0.14 |
b1b3770 GPU geri | 26def41 GPU 300W
```

## 5. AÇIK İŞLER (16 Eyl 17:10 — sahip + boyuta göre)

### BENDEN (onaysız başlayabilirim)
- B1. Üretim izleme: newscrawl kapanışlarında spot/escrow/ledger zinciri (pasif).
- B2. Site Y1/Y3/Y5 uygulama (Y2+Y4 kelime onayı gelirse; site push token ister).
- B3. komuta integration testleri (batch/escrow akışı `:memory:` DB'de).
- B4. Ölü `izle*.sh` temizliği + DURUM periyodik tazeleme.

### SENDEN (operatör)
- S1. Push tokeni (şu an 1 snapshot bekliyor).
- S2. CF WAF-skip kuralı (`/api/*`, 2 dk).
- S3. 4003 modem yönlendirme (tam mesh katılımı).
- S4. Gerçek dış makineden miner provası (WAF sonrası).
- S5. v1.2 onayı (imza belgen).
- S6. Site Y2+Y4 kelime onayı.
- S7. Avukat randevusu (0.9) + emanetçi/GDPR kararları.
- S8. Windows .exe/EV + H motoru (ilk ödeyen müşteri).

### BLOKELİ (dış bağımlılık)
- X1. wet_en (disk) / paracrawl (S3 404) / caselaw-full (hesap).
- X2. Bağımsız operatörler + topluluk kanalları.
- X3. Mainnet takvimi (Kasım: 1.1 sabitleri, genesis, wallet, explorer).

### KAPANDI (bu hafta)
cc100 109M ✅ | merge+FAISS-24 ✅ | arşiv 161G ✅ | B-1→B-5+BUG-1/2 ✅ |
E2E ✅ | localhost bind ✅ | miner UA ✅ | sıra timer ✅ | hafıza+tatbikat ✅ |
localhost-only ✅ | Dalga A (caselaw→newscrawl kesimi) ✅ | site miner paketi ✅

## 6. KARAR DEFTERİ (değişmez kayıt)

- **MİSYON (16 Eyl, operatör beyanı): KENDİ COİN + MERKEZİYETSİZ YAPAY ZEKA AĞI.**
  Tüm işler buna hizmet eder: coin tarafı (TOKENOMI kilitli → mainnet 1.1 →
  genesis → wallet), ağ tarafı (P2P → tohum-0 → bağımsız operatörler →
  kurucu-anahtar devri). Madencilik odağı bu misyonun Faz 0 adımıdır.

- Tokenomik KİLİTLİ: coin+halving, tavan 210M, era1 800 mikro, HALVING 50B
  (testnet: 2000 mikro + 5M ile sürer; mainnet tek commit'le geçirilir).
- Tek hat Rust CLI; miner-api/komuta_api.py/Tauri donduruldu; 6 iskelet `_arsiv/`de.
- Damıtma ertelendi (Qwen3-LoRA kilit). Tavan kota ilkesi (§2b) kilitli.
- GPU elleme yok (operatör emri, 12 Eyl). Merkezi sunucu/VPS yok ilkesi.
- SPOT %10 (siteyle uyumlu). Eşik 0.98 (kod) vs site 0.99 → site düzeltilecek.
- Arşivde SİLME YOK ilkesi (taşıma var, silme yok).
- **Kapsam kilidi (16 Eyl): SADECE MADENCİLİK.** Model çıkarma/damıtma/HF işi
  ileri döneme ertelendi (1.4, 3.2). Odak: embed/kanıt/ödül/ağ. Bu kilit
  değişmeden `model/` dizinine ve model iddialarına (site Y2) dokunulmaz.
- Canlı komuta DB'ye ALTER/DROP YOK; sadece CREATE TABLE / ADD COLUMN
  (Garanti-3; migration'lar `komuta-rs/migrations/`da, boot'ta otomatik).

## 7. YENİ OTURUM AÇILIŞ PROTOKOLÜ (sırayla)

```
1. Bu dosyayı oku (§1 snapshot + §5 açık işler).
2. Hızlı durum: ./durum-anlik.sh  (servis+disk+git+llama+faiss, ~10sn)
3. Derin gerekirse: ALTYAPI-KAYIT.md §4 (faz günlüğü) + git log -5.
4. Reboot testi: uptime < 30dk VE llama 0/6 VE /tmp/emb_*.log yoksa → reboot
   olmuş: ~/RESUME-1SAAT.md sırası (1-güç: CPU performance+250W,
   2-llama 1241-1246 manuel+health, 3-worker resume) — ama embed işi YOKSA
   llama/worker şart değil, 1241 sorgu kapısı yeter.
5. Push bekleyen varsa (§1): token iste, §8'deki 3 satırla bas, fetch ile eşitle.
```

## 8. CREDENTIAL / TOKEN DİSİPLİNİ

- Token'lar dosyaya ASLA yazılmaz; sohbete yapıştırılır, tek komutluk kullanılır,
  `unset` edilir. Remote config'e gömülmez.
- Hesap: `nemes-xspace` (repo private, push yetkili). Tam sıra (3 satır):
  `export GH_TOKEN='<sohbetten>'`
  `git push "https://x-access-token:$GH_TOKEN@github.com/nemes-xspace/NemesXSpace.git" main`
  `git fetch "https://x-access-token:$GH_TOKEN@github.com/nemes-xspace/NemesXSpace.git" main:refs/remotes/origin/main && unset GH_TOKEN`

## 9. DİZİN SÖZLÜĞÜ

- `NemesXSpace/` → kod + bu dosya + DURUM/ALTYAPI/PROJE-TANIMI (git'li).
- `/srv/beyin/wiki/` → corpus DB'leri + FAISS + llama logları + bekçi.
- `/srv/beyin/kaynaklar/` → pipeline scriptleri + sira.log + faiss.log.
- `/home/d3str0y1ng/nemes-merge/` → merged DB + arsiv/ + {.log} kanıtlar.
- `/home/d3str0y1ng/nemes-testnet/` → komuta.db + bin/ + yedek/.
- `~/Belgeler/NEMES X The Sovereign.md` → dış denetim (14 Eyl).
- `~/RESUME-1SAAT.md` → reboot açılış sırası. `~/İndirilenler/*.json` → oturum kayıtları.

## 10. ANATOMİ (16 Eyl — 3 kollu tarama özeti, detay ALTYAPI §1)

### 10.1 Kod modülleri (yol | görev | durum)
- `miner-core/{mining(574),llama(277),shard(173),resources(56),depolama(260),lib}` → aktif
- `miner-cli/main.rs(604)` → komutlar wallet/pull/keygen/mine/status/chat | aktif (chat stub)
- `komuta-rs/main.rs(2647)` → 21 endpoint (aşağıda) | aktif
- `komuta-rs/migrations/001-012` → şema evrimi; 012_escrow güncel baş | aktif
- `miner-api/`, Tauri `miner/`, `komuta_api.py.donduruldu` → dondurulmuş
- `nemes-p2p/` → nemes-core (tipler+MASTER_PUBKEY) + nemes-p2p (GOREV_TOPIC)
  + nemes-cli; 6 iskelet `_arsiv/`de; NETWORK_ID=nemes-mainnet-v1
- `wiki/*.py` → parse(142)/parse_par(215)/embed(161)/embed_par(216)/merge_en/wikihow
- `kaynaklar/*.py` → komutan/sira/adapter/faiss_kur/faiss_hepsi/sorgu/sorgu_hepsi/
  merge_vektor/stats_uret/alto_cevir/en_merge + cc_tr/cc_en çekiciler

### 10.2 Komuta endpointleri (21)
kayit, heartbeat, gorev, kanit, status, bakiye, arz, ledger, komut,
denetim, denetim/sonuc, kanarya/kontrol, shard/ilan, shard, ara (POST+GET),
parca/tohum, parca/indir, depolama/yoklama[/sonuc/uret] + health.

### 10.3 Kilit sabitler
SPOT=%10 (blake3 deterministik) | EŞİK=0.98 | DENETIM_BATCH=5, ödül 50 mikro |
SLASH=2000, ITIBAR -25/+1, STRIKE_LIMIT=3 | HALVING testnet 5M (mainnet 50B) |
TABAN testnet 2000 mikro (mainnet 800) | TASK 20'lik/600sn | ÖLÜ 300sn |
SHARD 1800sn/100k | PARÇA 64MiB×3 | YOKLAMA 1800sn/1MiB/2-fail | KOTA 100GiB |
FAISS PQ32, nlist 512/1024, nprobe=128 | EMBED nomic-1.5 768d, int8, BATCH=16.

### 10.4 Veri haritası (boyut | rol)
wiki_en 38G, gut_en 46G, newscrawl 48G, cc100 symlink→118G merged,
cc100 shard 12×2.2G (26G, 16 Eyl arşivde), de 16G, ru 16G, fr 13G, arxiv 12G,
es 11G, ja 9.2G, ar 5.9G, pt 5.1G, zh 4.8G, tr 2.4G + diğer dillerin
emb-ara çıktıları ~80 parça. FAISS 24 adet 7.4G. done bayrağı 24 adet
(`kaynaklar/sira_*.done`). testnet bin/ 125M, yedek/ 21G (14 dosya).

### 10.5 Doküman güncellik
GÜNCEL: ALTYAPI-KAYIT, bu dosya. YARI: DURUM (başlık 08 Eyl).
BAYAT: 00-PROJE-TANIMI (28 Ağu), README (28 Ağu) — okunur ama karar için
ALTYAPI geçerlidir. docs/: TOKENOMI kilitli, GUVENLIK kör-ID+kanarya,
YOL-HARITASI v1.1, 2 manifesto adayı, odul-stratejisi ARSIV'de.

### 10.6 Bilinen açıklar (güncel 16 Eyl)
CANLIDA: B-1 escrow+%10 ✅ | BUG-1 index ✅ | B-3 legacy kalktı ✅ |
B-4 salt ✅ | B-5 hash pini ✅ | BUG-2 retry ✅ | parse resume ✅ |
STRICT_DENETIM env ✅ | localhost bind ✅ | miner UA ✅ | E2E provası ✅.
AÇIK: öz-denetim warn-only (STRICT=1 mainnet öncesi) | site 0.99 vs kod 0.98
(site düzeltilecek) | miner-api sabit JWT (dondurulmuş kodda, canlıda değil) |
TUI mock (site iddiasıyla çelişmemeli) | MASTER_PUBKEY default gömülü (env ile
ezilmiyor; testnet toleransı, mainnet öncesi env zorunlu yapılacak).

### 10.7 Servis envanteri (tam, 16 Eyl süpürme)
beyin-sunucu (llama başlatıcı) | beyin_bekci (15sn) + KillMode drop-in |
beyin_sira (timer 5dk + flock) | beyin_wikihow (wikihow 2 işçi resume) |
nemes-embed-bundle (:1251 self-contained 194M) | cloudflared (+günlük update timerı) |
komuta-backup (günlük, `nemes-testnet/scripts/backup-db.py`) |
nemes-izleme-hizli (15dk) + gunluk (06:00) | nemes-komuta/miner-a/b |
caddy (User=caddy, /etc/caddy/Caddyfile) | mini7-dongu (gece ajan turu) |
cron: fwupd-izle (06:00) + antikor-fp (Pzt 06:30). Timer'sız ölüler: `izle*.sh`
(tmux dönemi bitti, silinebilir aday).

### 10.8 Ev dizini haritası (ölçülü)
`BEYIN`→/srv/beyin link | `kalitim/` 1.6G (günlük sabah raporları + opencode geçmişi —
ikinci hafıza, okunmadı detayı) | `nemes-egitim/` 15G (eğitim BİTMİŞ: adapter
checkpoint'ler + yanit-final 07 Eyl) | `nemes-bundle/` 194M | `Scriptler/` 612K |
`tools/` qwen-agent | `snap/` 23G | `bin/` boş | islem/encoded/decoded = test artığı.

### 10.9 /srv/beyin derinlik (süpürme)
kaynaklar 149G (cc_en 111G!) | website 4.2G (video_frames_white 3.6G şişkin!) |
model-v01 1.2G (train.jsonl) | ozel/ pazarlama docs (REKLAM/X-POST/HF-PLAN/KIMLIK) |
testler/mini7 gece turu | beyin.db (bilgi/deneyim/hata/gorev/miner + FTS) |
ANAYASA **v4.0** (31 Ağu, 320 madde — DURUM'daki "v3.0" BAYAT) |
beyin.py + ingest_paket×9 | komuta_api.py.donduruldu mevcut.

### 10.10 Kod artıkları (süpürme)
- komuta sabitleri (tam): DEADLINE 600sn, ÖLÜ 300sn, HAVUZ_YENİLE 120sn,
  SHARD_SURE 1800sn, REPLİKA 3, ONARIM ödül 100, YOKLAMA 1MiB/1800sn/5dk,
  MIN_KOTA 100GiB, ARA k=5/20. `komut`: imha (kara+strike3+pay/coin0) /
  affet (strike0/itibar100). Heartbeat 4096 parça limiti.
- miner TUI **MOCK** (sabit 14.2/s demo); gerçek iş `--simple`da. Site "indir"
  iddiasıyla çelişmemeli.
- P2P: gossip derece 4/12, MAX_PEERS 100; ceremony: offline parola + AES-256-PBKDF2(2M).
- miner-api dondurulmuş AMA içinde sabit JWT secret dizesi var (canlıda değil).
- İkinci deploy profili: `komuta-rs/deploy/nemes-komuta-public.service`
  (~/nemes-public, Mem 12G, CPU %400) — tohum-0 aday konfigürasyonu.
- Miner token'ları ps çıktısında görünebilir (dosyadan okunuyor ama cmdline riski) —
  log hijyenine eklenecek madde.

## 11. KÖR NOKTA DERSLERİ (16 Eyl — bu hatalar tekrarlanmayacak)
1. Config dosyası okunmadan "biliyorum" denmez (tünel dersi).
2. Varsayım taşınmaz: eski nottaki cümle, canlı config ile doğrulanır.
3. İkinci profiller aranır (public.service, bundle :1251, mini7, wikihow işçileri).
4. Sürüm kayması kontrol edilir (ANAYASA v3/v4, site sayıları, sabitler).
5. Secret'lar ps/env/log'da aranır, dosyaya yazılmaz.
6. Ölü kod/script (izle*.sh, dondurulmuş API) envanterde "ölü" işaretlenir.
7. Tatbikat her büyük hafıza değişiminde tekrarlanır.

## 12. PARANOYAK PROTOKOLÜ (16 Eyl — Garanti-9)

> Kural: Söylenmeden bakılır. Her oturumda §12.1, her hafta §12.2.
> Bulgu = kanıt + kayıt + (gerekirse) tek komutluk onaya hazır çözüm.

### 12.1 Oturum başı (5 dk, `durum-anlik.sh` + şunlar)
- `systemctl --failed` → izleme dahil her failed birimi açıkla.
- `ps` secret taraması (`--token HEX`, `ghp_`, `.pem` yolları) — boolean, içerik basılmaz.
- Disk trendi: bir önceki snapshot'la karşılaştır, saatte >5G erime = alarm.
- Timer'lar: `list-timers` kaçırılmış tetik var mı?
- Canlı DB yazılabilirlik + cursor/escrow sayıları (kilitliyse busy-timeoutla tekrar).

### 12.2 Haftalık derin (30 dk)
- Yedek GERİ YÜKLEME provası (kopyaya restore + integrity + tablo sayımı).
- Cert bitişleri (`openssl`, <30 gün = alarm).
- Repo leak taraması (ghp/AKIA/PEM — node_modules hariç tutulup elle bakılır).
- UFW kuralları + dinleyen portlar diff'i (beklenmeyen port = alarm).
- FAISS/DB/symlink mevcudiyeti + bir spot sorgu.
- Log anomalisi: son 24s error/critical + bekçi rolling boşluğu + izleme ALARM.log.

### 12.3 A-demeden-B listesi (otomatik eşikler)
- / <40G → cargo clean + yedek rotasyonu öner (tek komut).
- /srv <100G → büyük iş (merge/FAISS) ONAYSIZ başlamaz.
- Backup >26saat eski → timer kontrolü.
- Kanarya sayısı değiştiyse → GUVENLIK sahibine sor.
- Yeni failed unit → sebebini bulmadan kapatma.
- `izleme.py` eşikleri (DISK 25/BEYIN 60/üretim 30dk/kuyruk 500/emanet 24sa) yılda 2 kez gözden geçir.

### 12.4 Hijyen kuralları
- Token/secret: dosyaya ASLA, ps/journal'a ASLA (env-file + 600), sohbette tek kullanımlık.
- Yıkıcı işlem öncesi: sayım + kopya + geri dönüş yolu yazılı.
- Config değişimi: önce `cat`, sonra drop-in, sonra reload+doğrula.
- Tatbikat: her büyük hafıza değişiminde sıfır-bağlam testi.
