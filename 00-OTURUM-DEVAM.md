# NEMES-X — OTURUM DEVAM DOSYASI (tek hafıza)

> **Amaç:** Token bitince / yeni profil açılınca kaldığımız yerden devam.
> Yeni oturumda İLK bunu oku, sonra §7'deki sırayı izle.
> **Kural:** Her ilerleme bitiminde bu dosyanın §1'i güncellenir
> (ALTYAPI-KAYIT.md Garanti-8). Tarihsiz satır bırakılmaz.

---

## 1. SON SNAPSHOT — 16 Eyl 2026 13:35 (+03, date ile doğrulandı)

| Kalem | Durum |
|---|---|
| Git | Son push: `b285836` ✅ | Bekleyen: `git status -sb` çıktısındaki
  `[önünde: N]` sayısı esastır (bu satırdaki sayı YAZILMAZ — commitlendikçe
  eskir; N'i `git log origin/main..HEAD --oneline` ile gör) |
| Denetim-2 | B-4 (salt) + STRICT_DENETIM + BUG-2 + B-5 + parse-resume **canlıda**
  (`618d628`, migration 013 uygulandı, test 12/12, 3/3 servis active) |
| cc100_tr | KAPANDI ✅ 109.271.443 vektör, merge 118G, FAISS-24 (4.1G+834M, ntotal doğrulandı) |
| Denetim düzeltmeleri | B-1 (escrow+%10) + BUG-1 + B-3 **canlıda** (migration 012 uygulandı, restart yapıldı) |
| Hafıza sistemi | Bu dosya + `durum-anlik.sh` + Garanti-8 ✅; anatomi §10 eklendi, tatbikat yapıldı |
| Arşiv | emb 12x7G + bak 25G → `/home/d3str0y1ng/nemes-merge/arsiv/` (109G, silinen yok) |
| Servisler | 6/6 active + beyin_sira (ikisi de betikte sorgulanır) |
| Llama | 1241 tekil `ok` (sorgu kapısı); 1242-1246 kapalı (görev yok) |
| Miner'lar | Sağlıklı polling, `gorev:bekleniyor` (corpus tr tükenik — bilinen) |
| Disk | /srv/beyin **211G** boş (%76; wiki dizini 506G bu diskte), / **80G** boş (%91) |
| FAISS | 24 index, toplam ~7.4G (cc100 ~4.9G faiss+ids, diğer 23 ~2.5G) |
| Yedek | komuta-backup timer günlük çalışıyor (saat ~00:1x bandı); son manuel:
  `yedek/komuta-2026-09-16-manuel-618d628.db` (1.5G) |
| E2E escrow | Tetikleyici: yeni corpus görevi dağıtımı (corpus tr tükenik olduğu için
  şu an yok; operatör yeni korpus açınca) → zincir: spot kanıt → batch kapanışı
  → escrow satırı → denetim geçişi → `ledger nedeni='escrow'` kontrolü |

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
  Arsiv: /home/d3str0y1ng/nemes-merge/arsiv/ (13 dosya, 109G)
  Komuta DB: /home/d3str0y1ng/nemes-testnet/komuta.db (1.5G)
  Komuta bin: /home/d3str0y1ng/nemes-testnet/bin/ (.bak'lar yanında)
  Test loglari: /home/d3str0y1ng/nemes-merge/{merge,verify,arsiv,faiss_cc100}.log
```

## 4. SON 8 COMMIT (alt-usta eski→yeni)

```
a02d493 hafiza (bu dosya+betik+garanti-8, PUSH BEKLIYOR) |
43ee3ec deploy kaydi (PUSH BEKLIYOR) |
144def0 B-1/BUG-1/B-3 kod (PUSH BEKLIYOR) |
b285836 arsiv (PUSH'LU) | c71fb4b cc100 kapanis (PUSH'LU) |
03e07c6 yas kapisi 1800sn | c3a91be yol haritasi 0.14 |
b1b3770 GPU geri | 26def41 GPU 300W
```

## 5. AÇIK İŞLER (öncelik sırasıyla)

1. **Push 3 commit** (`144def0`+`43ee3ec`+`a02d493`; URL-gömülü basılır, §8).
2. **E2E escrow kanıtı** — tetikleyici §1'de; zincir orada yazılı.
3. **Denetim listesi kalan:** B-4 (spot salt), B-5 (tedarik hash, llama.rs),
   BUG-2 (miner retry), parse-merge resume, öz-denetim engeli (mainnet öncesi).
4. **Site senkronu:** 109M/24 index/coin+halving/kanarya özeti yayınlanacak
   (dış denetim belgesi `~/Belgeler/NEMES X The Sovereign.md` gerekçe).
5. **Kurucu anahtarı + GDPR/silme tasarımı** (iç boşluk, acil).
6. **cc100 shard 12×2.2G akıbeti:** merge kaynağı olarak duruyor; öneri arşive
   taşıma (SİLME YOK ilkesi korunarak), karar operatörde.
7. Llama 1242-1246: yeni embed korpusu gelince RESUME sırasıyla kaldırılır.

## 6. KARAR DEFTERİ (değişmez kayıt)

- Tokenomik KİLİTLİ: coin+halving, tavan 210M, era1 800 mikro, HALVING 50B
  (testnet: 2000 mikro + 5M ile sürer; mainnet tek commit'le geçirilir).
- Tek hat Rust CLI; miner-api/komuta_api.py/Tauri donduruldu; 6 iskelet `_arsiv/`de.
- Damıtma ertelendi (Qwen3-LoRA kilit). Tavan kota ilkesi (§2b) kilitli.
- GPU elleme yok (operatör emri, 12 Eyl). Merkezi sunucu/VPS yok ilkesi.
- SPOT %10 (siteyle uyumlu). Eşik 0.98 (kod) vs site 0.99 → site düzeltilecek.
- Arşivde SİLME YOK ilkesi (taşıma var, silme yok).
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
cc100 shard 12×2.2G (26G, duruyor), de 16G, ru 16G, fr 13G, arxiv 12G,
es 11G, ja 9.2G, ar 5.9G, pt 5.1G, zh 4.8G, tr 2.4G + diğer dillerin
emb-ara çıktıları ~80 parça. FAISS 24 adet 7.4G. done bayrağı 24 adet
(`kaynaklar/sira_*.done`). testnet bin/ 125M, yedek/ 21G (14 dosya).

### 10.5 Doküman güncellik
GÜNCEL: ALTYAPI-KAYIT, bu dosya. YARI: DURUM (başlık 08 Eyl).
BAYAT: 00-PROJE-TANIMI (28 Ağu), README (28 Ağu) — okunur ama karar için
ALTYAPI geçerlidir. docs/: TOKENOMI kilitli, GUVENLIK kör-ID+kanarya,
YOL-HARITASI v1.1, 2 manifesto adayı, odul-stratejisi ARSIV'de.

### 10.6 Bilinen açıklar (denetim 16 Eyl, dosya:satır referanslı)
B-1 escrow ✅canlı | BUG-1 index ✅canlı | B-3 legacy ✅kalktı |
B-4 spot salt ⏳ | B-5 llama.rs hash yok ⏳ | BUG-2 miner retry yok ⏳ |
parse-merge yıkıcı ⏳ | öz-denetim warn-only ⏳ | site 0.99/0.98 + %10 uyumsuzluğu:
kod doğru (%10, 0.98), site düzeltilecek.
