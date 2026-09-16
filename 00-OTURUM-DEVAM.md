# NEMES-X — OTURUM DEVAM DOSYASI (tek hafıza)

> **Amaç:** Token bitince / yeni profil açılınca kaldığımız yerden devam.
> Yeni oturumda İLK bunu oku, sonra §8'deki sırayı izle.
> **Kural:** Her ilerleme bitiminde bu dosyanın §1'i güncellenir
> (ALTYAPI-KAYIT.md Garanti-8). Tarihsiz satır bırakılmaz.

---

## 1. SON SNAPSHOT — 16 Eyl 2026 13:30 (+03)

| Kalem | Durum |
|---|---|
| Git | `main...origin/main` **[önünde: 2]** — `144def0` + `43ee3ec` push bekliyor |
| cc100_tr | KAPANDI ✅ 109.271.443 vektör, merge 118G, FAISS-24 (4.1G+834M, ntotal doğrulandı) |
| Denetim düzeltmeleri | B-1 (escrow+%10) + BUG-1 + B-3 **canlıda** (migration 012 uygulandı, restart yapıldı) |
| Arşiv | emb 12x7G + bak 25G → `/home/d3str0y1ng/nemes-merge/arsiv/` (109G, silinen yok) |
| Servisler | 6/6 active (bekçi, komuta, miner-a/b, caddy, cloudflared) |
| Llama | 1241 tekil `ok` (sorgu kapısı); 1242-1246 kapalı (görev yok) |
| Miner'lar | Sağlıklı polling, `gorev:bekleniyor` (corpus tr tükenik — bilinen) |
| Disk | /srv/beyin **211G** boş (%76), / **80G** boş (%91) |
| Yedek | `yedek/komuta-2026-09-16-manuel-144def0.db` (1.5G) + günlük timer |
| E2E escrow | İlk gerçek görev dağıtımında kanıtlanacak (şu an dağıtılacak iş yok) |

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
26def41 GPU 250->300W | b1b3770 300->250W geri | c3a91be yol haritasi 0.14 |
03e07c6 yas kapisi 1800sn | c71fb4b cc100 kapanis (PUSH'LU) |
b285836 arsiv (PUSH'LU) | 144def0 B-1/BUG-1/B-3 kod (PUSH BEKLIYOR) |
43ee3ec deploy kaydi (PUSH BEKLIYOR)
```

## 5. AÇIK İŞLER (öncelik sırasıyla)

1. **Push 2 commit** (token istenecek; remote config'e dokunmadan URL ile basılır).
2. **E2E escrow kanıtı** — ilk görev dağıtımında: spot düşen kanıt → batch kapanışı →
   escrow satırı → denetim geçişi → `ledger nedeni='escrow'` kontrolü.
3. **Denetim listesi kalan:** B-4 (spot salt), B-5 (tedarik hash, llama.rs),
   BUG-2 (miner retry), parse-merge resume, öz-denetim engeli (mainnet öncesi).
4. **Site senkronu:** 109M/24 index/coin+halving/kanarya özeti yayınlanacak
   (dış denetim belgesi `~/Belgeler/NEMES X The Sovereign.md` gerekçe).
5. **Kurucu anahtarı + GDPR/silme tasarımı** (iç boşluk, acil).
6. Llama 1242-1246: yeni embed korpusu gelince RESUME sırasıyla kaldırılır.

## 6. KARAR DEFTERİ (değişmez kayıt)

- Tokenomik KİLİTLİ: coin+halving, tavan 210M, era1 800 mikro, HALVING 50B
  (testnet: 2000 mikro + 5M ile sürer; mainnet tek commit'le geçirilir).
- Tek hat Rust CLI; miner-api/komuta_api.py/Tauri donduruldu; 6 iskelet `_arsiv/`de.
- Damıtma ertelendi (Qwen3-LoRA kilit). Tavan kota ilkesi (§2b) kilitli.
- GPU elleme yok (operatör emri, 12 Eyl). Merkezi sunucu/VPS yok ilkesi.
- SPOT %10 (siteyle uyumlu). Eşik 0.98 (kod) vs site 0.99 → site düzeltilecek.
- Arşivde SİLME YOK ilkesi (taşıma var, silme yok).

## 7. YENİ OTURUM AÇILIŞ PROTOKOLÜ (sırayla)

```
1. Bu dosyayı oku (§1 snapshot + §5 açık işler).
2. Hızlı durum: ./durum-anlik.sh  (servis+disk+git+llama+faiss, ~10sn)
3. Derin gerekirse: ALTYAPI-KAYIT.md §4 (faz günlüğü) + git log -5.
4. Reboot olmuşsa (uptime küçük): ~/RESUME-1SAAT.md sırası
   (güç → llama → worker) — ama embed işi YOKSA llama şart değil.
5. Push bekleyen varsa (§1): token iste, URL-gömülü bas, fetch ile eşitle.
```

## 8. CREDENTIAL / TOKEN DİSİPLİNİ

- Token'lar dosyaya ASLA yazılmaz; sohbete yapıştırılır, tek komutluk kullanılır,
  `unset` edilir. Remote config'e gömülmez.
- Hesap: `nemes-xspace` (repo private, push yetkili). Kullanım:
  `git push "https://x-access-token:$GH_TOKEN@github.com/nemes-xspace/NemesXSpace.git" main`
  sonra aynı URL ile `git fetch ... main:refs/remotes/origin/main`.

## 9. DİZİN SÖZLÜĞÜ

- `NemesXSpace/` → kod + bu dosya + DURUM/ALTYAPI/PROJE-TANIMI (git'li).
- `/srv/beyin/wiki/` → corpus DB'leri + FAISS + llama logları + bekçi.
- `/srv/beyin/kaynaklar/` → pipeline scriptleri + sira.log + faiss.log.
- `/home/d3str0y1ng/nemes-merge/` → merged DB + arsiv/ + {.log} kanıtlar.
- `/home/d3str0y1ng/nemes-testnet/` → komuta.db + bin/ + yedek/.
- `~/Belgeler/NEMES X The Sovereign.md` → dış denetim (14 Eyl).
- `~/RESUME-1SAAT.md` → reboot açılış sırası. `~/İndirilenler/*.json` → oturum kayıtları.
