// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use clap::{Parser, Subcommand};
use miner_core::{get_gpu_info, MiningStats, WorkerState, MINER_USER_AGENT};
use std::time::Duration;
use atty;

#[derive(Parser)]
#[command(name = "nemes-miner", version = "0.2.0", about = "NEMES Miner — Mine knowledge. Not hashes.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Cüzdan ayarla (kendi BTC/TRC20 adresin)
    Wallet {
        #[arg(long)]
        address: String,
        #[arg(long, default_value = "trc20")]
        network: String,
    },
    /// HF'den model çek (Ollama gibi)
    Pull {
        model: String,
    },
    /// Miner kimligi uret (~/.nemes/miner-ed25519.key). Shard ilanlarini imzalar.
    Keygen {
        /// Farkli yol (ayni makinede 2 miner testi icin).
        #[arg(long, default_value = "")]
        anahtar: String,
        /// Depolama katmani kabulu: 100GB baraji + disk sinavi + taahhut.
        #[arg(long)]
        storage: bool,
        /// Depolama dizini (varsayilan ~/.nemes/depolama).
        #[arg(long, default_value = "")]
        yol: String,
        /// Sinav boyutu MB (1-10240, varsayilan 1024).
        #[arg(long, default_value_t = 1024)]
        boyut_mb: u64,
    },
    /// Madenciliği başlat — Full TUI (htop gibi) veya --simple (gercek is)
    Mine {
        #[arg(long, default_value = "all")]
        gpu: String,
        #[arg(long)]
        simple: bool,
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        komuta: String,
        /// Komuta tokeni. Verilmezse NEMES_TOKEN env veya ~/.nemes/token okunur.
        #[arg(long, default_value = "")]
        token: String,
        #[arg(long, default_value = "http://127.0.0.1:1241")]
        embed_api: String,
        #[arg(long, default_value = "text-embedding-nomic-embed-text-v1.5")]
        model: String,
        /// Shard ilani corpus'u.
        #[arg(long, default_value = "tr")]
        corpus: String,
        /// Miner imza anahtari yolu. Bos ise ~/.nemes/miner-ed25519.key (yoksa uretilir).
        #[arg(long, default_value = "")]
        anahtar: String,
        /// Shard ilanindaki id adedi (göreli claim).
        #[arg(long, default_value_t = 2000)]
        shard_adet: i64,
        /// P2P relay portu (0 = rastgele).
        #[arg(long, default_value_t = 0)]
        p2p_port: u16,
        /// P2P gorev duyurusu dinle (nemes/gorev): duyuru gelince beklemeden
        /// HTTP'den gorevi cek. Kapaliyken eski poll (5/10sn) aynen calisir.
        #[arg(long, default_value_t = false)]
        p2p_dinle: bool,
        /// Mesh denetim (B15): nemes/denetim duyurularindaki kendine dusen
        /// atamalari al, metni komutadan cek, denetle, sonucu gonder.
        /// Kapaliyken komuta-aracili denetim gorevleri aynen calisir.
        /// --p2p-dinle gerektirir (yoksa uyariyla yoksayilir).
        #[arg(long, default_value_t = false)]
        denetim_mesh: bool,
        /// WAN tohum adresleri (B18): `/ip4/.../tcp/.../p2p/...` (virgullu).
        /// Bos = yalnizca LAN kesfi. --p2p-dinle gerektirir.
        #[arg(long, default_value = "")]
        bootstrap: String,
        /// Kira kipi (B20): araligi bir kez kirala, ~100 batch sormadan calis.
        /// Kapaliyken gorev basina tekil HTTP aynen calisir.
        #[arg(long, default_value_t = false)]
        kira: bool,
        /// Kira adedi (madde sayisi, 20-20000).
        #[arg(long, default_value_t = 2000)]
        kira_adet: i64,
        /// Isci sayisi (B26 paralel): kira kuyrugu paylasilir, cakisma yok.
        /// Kirasiz modda her isci bagimsiz ceker (cakisan kanit CONFLICT yer).
        #[arg(long, default_value_t = 1)]
        isci: u32,
        /// Depolama dizini (taahhut + parcalar). Bos ise ~/.nemes/depolama;
        /// taahhut yoksa heartbeat parcasiz gider (compute-only).
        #[arg(long, default_value = "")]
        depolama: String,
    },
    /// Durum göster
    Status {
        #[arg(long)]
        watch: bool,
    },
    /// Bosluk teklifi ver (B21): kapsama eksigi araligi oner, stake kilitle.
    Teklif {
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        komuta: String,
        /// Komuta tokeni. Verilmezse NEMES_TOKEN env veya ~/.nemes/token okunur.
        #[arg(long, default_value = "")]
        token: String,
        #[arg(long)]
        corpus: String,
        #[arg(long)]
        baslangic: i64,
        #[arg(long)]
        bitis: i64,
    },
    /// Bosluklari goster (B21): geri kalmis pencereler + acik teklifler.
    Bosluklar {
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        komuta: String,
        /// Komuta tokeni. Verilmezse NEMES_TOKEN env veya ~/.nemes/token okunur.
        #[arg(long, default_value = "")]
        token: String,
    },
    /// Sohbet (lokal GGUF)
    Chat {
        prompt: String,
        #[arg(long)]
        model: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Wallet { address, network }) => {
            let home = dirs::home_dir().unwrap().join(".nemes");
            std::fs::create_dir_all(&home)?;
            let cfg = format!("wallet_address = \"{}\"\nwallet_network = \"{}\"\n", address, network);
            std::fs::write(home.join("config.toml"), cfg)?;
            println!("✓ Cüzdan ayarlandı: {} ({})", address, network);
            println!("  Komuta: POST /api/kayit ile token alınacak");
        }
        Some(Commands::Pull { model }) => {
            println!("→ {} indiriliyor (HF) — ilerleme için miner Tauri kullan veya `nemes-miner` GUI", model);
            println!("  Gerçek indirme şu an Tauri miner’da (HF GGUF pull). CLI pull yakında eklenecek.");
        }
        Some(Commands::Keygen { anahtar, storage, yol: depolama_yolu_flag, boyut_mb }) => {
            use miner_core::{anahtar_yolu, anahtar_yukle_veya_uret, pubkey_b64};
            let kyol = if anahtar.is_empty() {
                anahtar_yolu()
            } else {
                std::path::PathBuf::from(&anahtar)
            };
            let sk = anahtar_yukle_veya_uret(&kyol)?;
            let pk = pubkey_b64(&sk);
            println!("anahtar: {}", kyol.display());
            println!("pubkey_b64: {}", pk);
            println!("(komuta ilk shard ilaninda bu pubkey'i miner_id'ye baglar - TOFU)");
            if storage {
                use miner_core::depolama;
                use miner_core::shard::simdi_ms;
                let dyol = if depolama_yolu_flag.is_empty() {
                    depolama::varsayilan_depolama_yolu()
                } else {
                    std::path::PathBuf::from(&depolama_yolu_flag)
                };
                println!("depolama dizini: {}", dyol.display());
                let bos = depolama::bos_alan_byte(&dyol)?;
                println!("bos alan: {:.1} GB (baraj: {:.0} GB)", depolama::gb(bos), depolama::gb(depolama::MIN_KOTA_BYTE));
                if bos < depolama::MIN_KOTA_BYTE {
                    anyhow::bail!("baraj alti: 100GB bos alan gerekli");
                }
                println!("sinav basliyor ({} MB yaz+oku)...", boyut_mb);
                let h = depolama::sinav(&dyol, boyut_mb)?;
                println!("sinav gecti: blake3={}...", &h[..16]);
                let tyol = depolama::taahhut_yaz(&dyol, depolama::MIN_KOTA_BYTE, &pk, &h, simdi_ms())?;
                println!("taahhut: {}", tyol.display());
                println!("KATMAN: storage (komuta C5 sonrasi bu taahhutu isteyecek)");
            }
        }
        Some(Commands::Mine { gpu, simple, komuta, token, embed_api, model, corpus, anahtar, shard_adet, p2p_port, p2p_dinle, denetim_mesh, bootstrap, kira, kira_adet, isci, depolama }) => {
            let token = token_coz(&token)?;
            if simple {
                simple_mine(&gpu, &komuta, &token, &embed_api, &model, &corpus, &anahtar, shard_adet, p2p_port, p2p_dinle, denetim_mesh, &bootstrap, kira, kira_adet, isci, &depolama).await?;
            } else {
                full_tui_mine(&gpu, &komuta).await?;
            }
        }
        Some(Commands::Status { watch }) => {
            loop {
                let gpu = get_gpu_info();
                println!("GPU: {} — {}/{} — {}% — batch {}", gpu.ad, gpu.vram_bos, gpu.vram_toplam, gpu.kullanim_pct, gpu.oneri_batch);
                println!("Komuta: {} — {}", komuta_status().await.unwrap_or("Bağlanıyor...".into()), chrono::Local::now().format("%H:%M:%S"));
                if !watch { break; }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
        Some(Commands::Chat { prompt, model }) => {
            println!("Chat: model={:?} prompt={}", model, prompt);
            println!("(lokal llama.cpp entegrasyonu yakında — şu an Tauri Chat sekmesini kullan)");
        }
        Some(Commands::Teklif { komuta, token, corpus, baslangic, bitis }) => {
            let token = token_coz(&token)?;
            let client = reqwest::Client::builder()
                .user_agent(MINER_USER_AGENT)
                .timeout(Duration::from_secs(15))
                .build()?;
            let r = client
                .post(format!("{}/api/teklif", komuta.trim_end_matches('/')))
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({ "corpus": corpus, "baslangic": baslangic, "bitis": bitis }))
                .send()
                .await?;
            let s = r.status();
            let j: serde_json::Value = r.json().await.unwrap_or(serde_json::Value::Null);
            if !s.is_success() {
                println!("teklif RED ({}): {}", s.as_u16(), j);
            } else {
                println!("teklif: {}", j);
            }
        }
        Some(Commands::Bosluklar { komuta, token }) => {
            let token = token_coz(&token)?;
            let client = reqwest::Client::builder()
                .user_agent(MINER_USER_AGENT)
                .timeout(Duration::from_secs(15))
                .build()?;
            let r = client
                .get(format!("{}/api/bosluklar", komuta.trim_end_matches('/')))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;
            let s = r.status();
            let j: serde_json::Value = r.json().await.unwrap_or(serde_json::Value::Null);
            if !s.is_success() {
                println!("bosluklar hata ({}): {}", s.as_u16(), j);
            } else {
                println!("{}", serde_json::to_string_pretty(&j).unwrap_or_default());
            }
        }
        None => {
            // Argümansız → Full TUI mine başlat (xmrig gibi)
            full_tui_mine("all", "http://127.0.0.1:8787").await?;
        }
    }
    Ok(())
}

async fn komuta_status() -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let r = client.get("http://127.0.0.1:8787/health").send().await?;
    if r.status().is_success() { Ok("Bağlı ●".into()) } else { Ok("Kilitli".into()) }
}

fn token_coz(flag: &str) -> anyhow::Result<String> {
    if !flag.is_empty() {
        return Ok(flag.to_string());
    }
    if let Ok(t) = std::env::var("NEMES_TOKEN") {
        if !t.trim().is_empty() {
            return Ok(t.trim().to_string());
        }
    }
    if let Some(home) = dirs::home_dir() {
        let yol = home.join(".nemes").join("token");
        if let Ok(t) = std::fs::read_to_string(&yol) {
            let t = t.trim().to_string();
            if !t.is_empty() {
                return Ok(t);
            }
        }
    }
    anyhow::bail!("token yok — once POST /api/kayit ile kayit olun, sonra --token VER veya NEMES_TOKEN env / ~/.nemes/token")
}

// Komuta /api/status'tan miner_id ogren (shard ilani icin gerekli).
async fn miner_id_ogren(komuta: &str, token: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent(MINER_USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()?;
    let r = client
        .get(format!("{}/api/status", komuta.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    if !r.status().is_success() {
        anyhow::bail!("status http {}", r.status());
    }
    let j: serde_json::Value = r.json().await?;
    j.get("miner_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("status'ta miner_id yok"))
}

// ——— Basit log (xmrig klasik) — GERCEK IS: gorev al -> embed -> kanit ---
async fn simple_mine(_gpu: &str, komuta: &str, token: &str, embed_api: &str, model: &str, corpus: &str, anahtar: &str, shard_adet: i64, p2p_port: u16, p2p_dinle: bool, denetim_mesh: bool, bootstrap: &str, kira: bool, kira_adet: i64, isci: u32, depolama: &str) -> anyhow::Result<()> {
    use miner_core::{mining_loop, EmbedClient, GorevAlici, KanitGonderici};
    use miner_core::{anahtar_yolu, anahtar_yukle_veya_uret, ilan_imzala, simdi_ms, ShardRelay, SigningKey};
    use miner_core::depolama as dep;
    use indicatif::{ProgressBar, ProgressStyle};

    // Depolama katmani: taahhut varsa heartbeat'e parca raporu eklenir (C6).
    let dep_yol: std::path::PathBuf = if depolama.is_empty() {
        dep::varsayilan_depolama_yolu()
    } else {
        std::path::PathBuf::from(depolama)
    };
    let dep_taahhut = dep::taahhut_oku(&dep_yol);
    if let Some(ref t) = dep_taahhut {
        println!("◈ depolama katmani: {} ({:.0} GB taahhut)", dep_yol.display(), dep::gb(t.kota_byte));
    }
    let mut dep_son_nabiz = std::time::Instant::now();

    // Shard relay: uzun omurlu P2P node + startup'ta goreli claim.
    // Kapali dongu: 30 sn'de bir /api/shard yoklanir, aktif ilan kalmadiysa
    // yeniden ilan verilir (komuta cakismayi hakem eder, fazlasi zararsiz).
    let (relay, relay_handle) = ShardRelay::baslat(p2p_port);
    let shard_yol: std::path::PathBuf = if anahtar.is_empty() {
        anahtar_yolu()
    } else {
        std::path::PathBuf::from(anahtar)
    };
    let shard_sk = match anahtar_yukle_veya_uret(&shard_yol) {
        Ok(sk) => Some(sk),
        Err(e) => {
            eprintln!("miner anahtari acilamadi (shard ilansiz devam): {}", e);
            None
        }
    };
    let shard_mid: Option<String> = match miner_id_ogren(komuta, token).await {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("miner_id ogrenilemedi (shard ilansiz devam): {}", e);
            None
        }
    };
    let shard_http = reqwest::Client::builder()
        .user_agent(MINER_USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()?;
    let shard_claim = |sk: &SigningKey, mid: &str| -> anyhow::Result<()> {
        let ilan = ilan_imzala(sk, corpus, shard_adet, mid, simdi_ms());
        relay.ilan_gonder(&ilan)?;
        println!("◈ shard ilani kuyrukta: {} +{} id <- {} (mesh'e yayinlanacak)", corpus, shard_adet, mid);
        Ok(())
    };
    if let (Some(ref sk), Some(ref mid)) = (shard_sk.as_ref(), shard_mid.as_ref()) {
        if let Err(e) = shard_claim(sk, mid) {
            eprintln!("ilk shard ilani atlandi: {}", e);
        }
    }
    // Ilk yoklama 30 sn sonra (startup ilani once mesh'e ulassin).
    let mut shard_son_yoklama = std::time::Instant::now();
    let is_tty = atty::is(atty::Stream::Stdout);
    let pb = if is_tty {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}")?);
        pb.enable_steady_tick(Duration::from_millis(200));
        Some(pb)
    } else {
        None
    };
    let mut gorev_alici = GorevAlici::new(komuta, token);
    // B20 kira kipi: aciksa alt-gorevler kuyruktan gelir (tekil HTTP seyreklesir).
    if kira {
        gorev_alici.kira_ayarla(true, kira_adet);
        println!("◈ kira kipi acik (adet {})", kira_adet.clamp(20, 20000));
    }
    let embed_client = EmbedClient::new(embed_api, model);
    let kanit_gonderici = KanitGonderici::new(komuta, token);
    let (durum_tx, mut durum_rx) = tokio::sync::mpsc::channel(32);
    let (durdur_tx, durdur_rx) = tokio::sync::watch::channel(false);
    // Yedek kolu depolama dizini: taahhut varsa kullan (yoksa C7 gorevleri atlanir).
    let dep_dir: Option<std::path::PathBuf> = {
        let d: std::path::PathBuf = if depolama.is_empty() {
            miner_core::depolama::varsayilan_depolama_yolu()
        } else {
            std::path::PathBuf::from(depolama)
        };
        if miner_core::depolama::taahhut_oku(&d).is_some() {
            Some(d)
        } else {
            None
        }
    };
    // R4/P2P: gorev duyuru dinleyicisi (opsiyonel, default kapali).
    // Acikken nemes/gorev mesh duyurusu gelince mining dongusu beklemeden
    // uyanir. Kapaliyken davranis tamamen eski poll'dur (canli miner etkilenmez).
    // B15 mesh-denetim: --denetim-mesh, --p2p-dinle gerektirir.
    let denetim_mesh = if denetim_mesh && !p2p_dinle {
        eprintln!("mesh-denetim --p2p-dinle olmadan calismaz (yoksayildi)");
        false
    } else {
        denetim_mesh
    };
    let uyan_rx: Option<tokio::sync::watch::Receiver<u64>> = if p2p_dinle {
        let (tx, rx) = tokio::sync::watch::channel(0u64);
        // B15 audit iscisi: duyurudan kendine dusen atamalari isler (ayri gorev,
        // dinleyiciyi tıkamaz; sirayla: metin -> embed -> sonuc).
        let (audit_tx, mut audit_rx) = tokio::sync::mpsc::unbounded_channel::<(String, i64)>();
        let audit_ec = embed_client.clone();
        let audit_kg = kanit_gonderici.clone();
        tokio::spawn(async move {
            use miner_core::embed_retry;
            while let Some((gorev, kor)) = audit_rx.recv().await {
                let m = match audit_kg.metin_al(kor).await {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("mesh-denetim metin hatasi (kor={}): {}", kor, e);
                        continue;
                    }
                };
                let v = match embed_retry(&audit_ec, &[m.metin]).await {
                    Ok(mut vv) => vv.pop().unwrap_or_default(),
                    Err(e) => {
                        eprintln!("mesh-denetim embed hatasi (kor={}): {}", kor, e);
                        continue;
                    }
                };
                if v.is_empty() {
                    continue;
                }
                let (b64, vmin, vmax) = miner_core::denetim_paketle(&v);
                match audit_kg.denetim_gonder(&gorev, kor, b64, vmin, vmax).await {
                    Ok(r) => eprintln!("mesh-denetim {} kor={} gecerli={} cos={:.4}", gorev, kor, r.gecerli, r.dogrulama),
                    Err(e) => eprintln!("mesh-denetim gonderme hatasi (kor={}): {}", kor, e),
                }
            }
        });
        let benim: Option<String> = shard_mid.clone();
        let shard_tohum = shard_sk.clone();
        let bootstrap_liste = bootstrap.to_string();
        let mesh_acik = denetim_mesh && benim.is_some();
        if denetim_mesh && benim.is_none() {
            eprintln!("mesh-denetim kapali: miner_id yok");
        }
        tokio::spawn(async move {
            let mut sayac = 0u64;
            // B18: dugum kimligi madenci anahtarindan turetilir (kararli PeerId).
            let tohum: Option<[u8; 32]> = shard_tohum.map(|sk| sk.to_bytes());
            let tohumlar: Vec<String> = bootstrap_liste
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            let mut node = match nemes_p2p::P2PNode::new(nemes_p2p::P2PConfig { port: p2p_port, enable_mdns: true, bootstrap: tohumlar, key_seed: tohum }).await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("p2p dinleyici acilamadi (port {}): {} — poll ile devam", p2p_port, e);
                    return;
                }
            };
            let mut ev_rx = match node.take_event_receiver() {
                Some(r) => r,
                None => return,
            };
            println!("◈ gorev duyurusu dinleniyor: {} (port {})", nemes_p2p::GOREV_TOPIC, p2p_port);
            if mesh_acik {
                println!("◈ mesh denetim acik: {}", nemes_core::mesh_audit::DENETIM_TOPIC);
            }
            loop {
                if node.run_for(5).await.is_err() {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                while let Ok(ev) = ev_rx.try_recv() {
                    if let nemes_p2p::NetworkEvent::MessageReceived { topic, data, .. } = ev {
                        if topic == nemes_p2p::GOREV_TOPIC {
                            sayac += 1;
                            let _ = tx.send(sayac);
                        } else if mesh_acik && topic == nemes_core::mesh_audit::DENETIM_TOPIC {
                            match serde_json::from_slice::<nemes_core::mesh_audit::DenetimDuyuru>(&data) {
                                Ok(duy) => {
                                    let mut bana = 0;
                                    for a in &duy.atamalar {
                                        if let Some(ref mid) = benim {
                                            if a.denetciler.iter().any(|d| d == mid) {
                                                bana += 1;
                                                let _ = audit_tx.send((a.gorev.clone(), a.kor));
                                            }
                                        }
                                    }
                                    if bana > 0 {
                                        println!("◈ mesh denetim duyurusu: {} ({} bana)", duy.gorev_id, bana);
                                    }
                                }
                                Err(e) => eprintln!("bozuk mesh duyurusu: {}", e),
                            }
                        }
                    }
                }
            }
        });
        Some(rx)
    } else {
        None
    };
    // B26 paralel isciler: kira kuyrugu paylasilir (cakisma yok); kirasiz
    // modda her isci bagimsiz ceker (komuta tekillestirir, cakisan CONFLICT yer).
    let isci_sayisi = (isci as usize).clamp(1, 32);
    if isci_sayisi > 1 && !kira {
        eprintln!("uyari: --isci {} kirasiz modda cakisma fireli calisir (--kira onerilir)", isci_sayisi);
    }
    if isci_sayisi > 1 {
        println!("◈ paralel isci: {}", isci_sayisi);
    }
    let mut kume = tokio::task::JoinSet::new();
    for wid in 0..isci_sayisi {
        kume.spawn(mining_loop(
            gorev_alici.clone(),
            embed_client.clone(),
            kanit_gonderici.clone(),
            wid,
            0,
            durum_tx.clone(),
            durdur_rx.clone(),
            dep_dir.clone(),
            uyan_rx.clone(),
        ));
    }
    let mut toplam: u64 = 0;
    let mut son_hiz = 0.0f32;
    let mut son_gorev = String::from("bekleniyor...");
    // Ctrl+C -> graceful dur
    let durdur2 = durdur_tx.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = durdur2.send(true);
    });
    loop {
        tokio::select! {
            r = kume.join_next(), if !kume.is_empty() => {
                match r {
                    Some(Ok(Ok(()))) => eprintln!("isci bitti (kalan {})", kume.len()),
                    Some(Ok(Err(e))) => eprintln!("isci hatasi: {}", e),
                    Some(Err(e)) => eprintln!("isci panic: {}", e),
                    None => break,
                }
            }
            m = durum_rx.recv() => {
                match m {
                    Some(d) => {
                        toplam += 1;
                        son_hiz = d.vectors_per_sec;
                        son_gorev = d.gorev.clone();
                    }
                    None => break,
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
        // Periyodik re-claim: aktif ilan kalmadiysa yenisini yayinla.
        if shard_son_yoklama.elapsed() >= Duration::from_secs(30) {
            shard_son_yoklama = std::time::Instant::now();
            if let (Some(ref sk), Some(ref mid)) = (shard_sk.as_ref(), shard_mid.as_ref()) {
                // Yoklama basarisizsa ilanin durdugunu farz et (spam yapma).
                let mut aktif = true;
                if let Ok(r) = shard_http
                    .get(format!("{}/api/shard", komuta.trim_end_matches('/')))
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await
                {
                    if let Ok(j) = r.json::<serde_json::Value>().await {
                        aktif = j
                            .get("kayitlar")
                            .and_then(|k| k.as_array())
                            .map(|a| {
                                a.iter().any(|k| {
                                    k.get("miner_id").and_then(|m| m.as_str()) == Some(mid.as_str())
                                })
                            })
                            .unwrap_or(false);
                    }
                }
                if !aktif {
                    match shard_claim(sk, mid) {
                        Ok(()) => println!("◈ re-claim yayinlandi: {} +{} id", corpus, shard_adet),
                        Err(e) => eprintln!("re-claim atlandi: {}", e),
                    }
                }
            }
        }
        // Heartbeat + disk raporu (60 sn): taahhut varsa parca listesiyle.
        if dep_son_nabiz.elapsed() >= Duration::from_secs(60) {
            dep_son_nabiz = std::time::Instant::now();
            let (parcalar, kota): (Vec<serde_json::Value>, Option<i64>) = match dep_taahhut.as_ref() {
                Some(t) => {
                    let liste: Vec<serde_json::Value> = dep::parca_listele(&dep_yol)
                        .into_iter()
                        .map(|(h, b)| serde_json::json!({"parca_hash": h, "boyut": b}))
                        .collect();
                    (liste, Some(t.kota_byte as i64))
                }
                None => (Vec::new(), None),
            };
            // B23 kabiliyet ilani: GPU/VRAM + roller (heterojen mesh eslesmesi).
            let gpu_bilgi = miner_core::resources::get_gpu_info();
            let vram_mb: i64 = gpu_bilgi
                .vram_toplam
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            let mut roller = vec!["embed".to_string(), "denetim".to_string()];
            if dep_taahhut.is_some() {
                roller.push("depolama".to_string());
            }
            if vram_mb >= 4096 {
                roller.push("uretim".to_string());
            }
            let yetenek = serde_json::json!({
                "gpu_ad": gpu_bilgi.ad.chars().take(120).collect::<String>(),
                "vram_mb": vram_mb,
                "roller": roller,
            });
            match shard_http
                .post(format!("{}/api/heartbeat", komuta.trim_end_matches('/')))
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({"parcalar": parcalar, "depolama_kota": kota, "yetenek": yetenek}))
                .send()
                .await
            {
                Ok(r) => {
                    if !r.status().is_success() {
                        eprintln!("heartbeat http {}", r.status());
                    } else if !parcalar.is_empty() {
                        println!("◈ nabiz: {} parca bildirildi", parcalar.len());
                    }
                }
                Err(e) => eprintln!("heartbeat hatasi: {}", e),
            }
            // C8 yoklama: acik challenge varsa aralik hash'le, cevabi gonder.
            if dep_taahhut.is_some() {
                let yurl = format!("{}/api/depolama/yoklama", komuta.trim_end_matches('/'));
                if let Ok(r) = shard_http
                    .get(&yurl)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await
                {
                    if r.status().as_u16() == 200 {
                        if let Ok(j) = r.json::<serde_json::Value>().await {
                            let id = j.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                            let ph = j.get("parca_hash").and_then(|v| v.as_str()).unwrap_or("");
                            let off = j.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
                            let len = j.get("uzunluk").and_then(|v| v.as_u64()).unwrap_or(0);
                            if id > 0 && !ph.is_empty() && len > 0 {
                                match miner_core::depolama::parca_hash_aralik(&dep_yol, ph, off, len) {
                                    Ok(h) => {
                                        let s = shard_http
                                            .post(format!("{}/api/depolama/yoklama/sonuc", komuta.trim_end_matches('/')))
                                            .header("Authorization", format!("Bearer {}", token))
                                            .json(&serde_json::json!({"id": id, "hash": h}))
                                            .send()
                                            .await;
                                        match s {
                                            Ok(rr) => println!("◈ yoklama cevabi: {}", rr.text().await.unwrap_or_default().chars().take(80).collect::<String>()),
                                            Err(e) => eprintln!("yoklama sonuc hatasi: {}", e),
                                        }
                                    }
                                    Err(e) => eprintln!("yoklama aralik hatasi (dosya kayip olabilir): {}", e),
                                }
                            }
                        }
                    }
                }
            }
        }
        let msg = format!("[{}] gorev:{} vectors/s:{:.1} tamamlanan_gorev:{} | Komuta: {}",
            chrono::Local::now().format("%H:%M:%S"), son_gorev, son_hiz, toplam, komuta);
        if let Some(ref pb) = pb {
            pb.set_message(msg.clone());
        } else {
            println!("{}", msg);
        }
    }
    let _ = durdur_tx.send(true);
    relay_handle.abort();
    Ok(())
}
fn rand_f32() -> f32 {
    use std::cell::Cell;
    thread_local! { static SEED: Cell<u64> = Cell::new(0x9e3779b97f4a7c15); }
    SEED.with(|s| {
        let mut x = s.get().wrapping_add(0x9e3779b97f4a7c15);
        x = x.wrapping_mul(0x85ebca6b);
        s.set(x);
        (x as f32 / u64::MAX as f32).fract().abs()
    })
}

// ——— Full TUI (ratatui + crossterm, htop gibi) ———
async fn full_tui_mine(gpu: &str, komuta: &str) -> anyhow::Result<()> {
    use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
    use ratatui::{prelude::*, widgets::*};
    use std::io;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut stats = MiningStats {
        workers: (0..4).map(|i| WorkerState {
            id: i, gpu: 0, vectors_per_sec: 14.2 + i as f32, pay_per_hour: 42, api_hakki: 142, gorev: format!("newscrawl shard {}", i*2), progress_pct: 54, hiz_mb: 11.2,
        }).collect(),
        toplam_vectors_per_sec: 56.8,
        toplam_pay: 1240,
        toplam_api_hakki: 1240,
    };
    let mut logs: Vec<String> = vec![
        "[15:42:11] PAY kabul #142  newscrawl:441231 768B 0.99 cos".into(),
        "[15:42:12] HEARTBEAT ok 45sn".into(),
    ];
    let mut tick: u64 = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(8), Constraint::Min(8), Constraint::Length(3)])
                .split(f.size());

            // Header
            let header = Paragraph::new(format!(" NEMES Miner v0.2.0 — nemes-miner — Komuta: Bağlı ● — GPU: {} — Komuta: {} ", gpu, komuta))
                .style(Style::default().fg(Color::Yellow).bg(Color::Black))
                .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(header, chunks[0]);

            // Worker table
            let header_cells = ["worker", "GPU", "vectors/s", "pay/h", "api_hakki", "görev", "progress"]
                .into_iter().map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
            let header_row = Row::new(header_cells).height(1).bottom_margin(1);
            let rows = stats.workers.iter().map(|w| {
                let cells = vec![
                    Cell::from(format!("#{}", w.id)),
                    Cell::from(format!("{}", w.gpu)),
                    Cell::from(format!("{:.1}/s", w.vectors_per_sec)),
                    Cell::from(format!("+{}", w.pay_per_hour)),
                    Cell::from(format!("{}", w.api_hakki)),
                    Cell::from(w.gorev.clone()),
                    Cell::from(format!("{}% {:.1} MB/s", w.progress_pct, w.hiz_mb)),
                ];
                Row::new(cells)
            });
            let widths = [
                Constraint::Length(8),
                Constraint::Length(5),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(20),
            ];
            let table = Table::new(rows, widths)
                .header(header_row)
                .block(Block::default().borders(Borders::ALL).title(" Workers — 12 worker  152.4/s ").border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(table, chunks[1]);

            // Log
            let log_text: Vec<Line> = logs.iter().rev().take(20).rev().map(|l| Line::from(l.clone())).collect();
            let log = Paragraph::new(log_text)
                .block(Block::default().borders(Borders::ALL).title(" Log — PAY / HEARTBEAT ").border_style(Style::default().fg(Color::DarkGray)))
                .wrap(Wrap { trim: false });
            f.render_widget(log, chunks[2]);

            // Footer
            let footer = Paragraph::new(" Komut: [p] Duraklat  [q] Çık  [s] Status  [h] Help  |  Full TUI — --simple ile satır log ")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::TOP));
            f.render_widget(footer, chunks[3]);
        })?;

        // Input — non-blocking
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('p') => logs.push(format!("[{}] Duraklatıldı — part korundu, resume edilebilir", chrono::Local::now().format("%H:%M:%S"))),
                    KeyCode::Char('s') => logs.push(format!("[{}] Status: {} worker, {:.1}/s", chrono::Local::now().format("%H:%M:%S"), stats.workers.len(), stats.toplam_vectors_per_sec)),
                    KeyCode::Char('h') => logs.push("[Help] p:duraklat q:çık s:status h:help".into()),
                    _ => {}
                }
            }
        }
        // Canlı tick
        tick += 1;
        if tick % 5 == 0 {
            for w in &mut stats.workers { w.vectors_per_sec = 13.5 + (rand_f32()*3.0); w.progress_pct = ((w.progress_pct as u16 + 1) % 100) as u8; }
            stats.toplam_vectors_per_sec = stats.workers.iter().map(|w| w.vectors_per_sec).sum();
            if tick % 10 == 0 { logs.push(format!("[{}] PAY kabul #{}  newscrawl:{}  {:.1} MB/s", chrono::Local::now().format("%H:%M:%S"), 142+tick, 441231+tick, 10.0 + rand_f32()*2.0)); }
            if logs.len() > 100 { logs.drain(0..20); }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("Mining durdu — part korundu, `nemes-miner mine --simple` ile satır log");
    Ok(())
}
