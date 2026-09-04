// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use anyhow::{anyhow, Result};
use std::process::Child;
use reqwest::blocking;
use zip::ZipArchive;

/// llama.cpp binary yollarını yöneten wrapper
pub struct LlamaServer {
    binary_path: PathBuf,
    child: Option<Child>,
    port: u16,
    host: String,
    model_path: PathBuf,
    n_gpu_layers: i32,
    ctx_size: usize,
    n_threads: usize,
}

impl LlamaServer {
    pub fn new(model_path: impl Into<PathBuf>, port: u16) -> Self {
        let model_path: PathBuf = model_path.into();
        Self {
            binary_path: Self::binary_path(),
            child: None,
            port,
            host: "127.0.0.1".to_string(),
            model_path,
            n_gpu_layers: 99,  // tüm layer'ları GPU'ya
            ctx_size: 2048,
            n_threads: 4,
        }
    }

    /// Binary yolu - önce local `binaries/` sonra PATH'te ara
    fn binary_path() -> PathBuf {
        // 1. Proje içi binaries/ klasörü
        let local = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries").join(Self::binary_name());
        if local.exists() {
            return local;
        }
        // 2. Cargo target/release (dev modunda)
        let cargo_target = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/release")
            .join(Self::binary_name());
        if cargo_target.exists() {
            return cargo_target;
        }
        // 3. PATH'te ara
        if let Ok(path) = which::which(Self::binary_name()) {
            return path;
        }
        // 4. Default - kullanıcı PATH'te bulacak
        PathBuf::from(Self::binary_name())
    }

    #[cfg(target_os = "windows")]
    fn binary_name() -> &'static str {
        "llama-server.exe"
    }

    #[cfg(not(target_os = "windows"))]
    fn binary_name() -> &'static str {
        "llama-server"
    }

    /// GPU layer sayısı ayarla
    pub fn with_gpu_layers(mut self, n: i32) -> Self {
        self.n_gpu_layers = n;
        self
    }

    pub fn with_ctx_size(mut self, size: usize) -> Self {
        self.ctx_size = size;
        self
    }

    pub fn with_threads(mut self, n: usize) -> Self {
        self.n_threads = n;
        self
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sunucuyu başlat
    pub fn start(&mut self) -> Result<()> {
        if self.child.is_some() {
            return Err(anyhow!("Sunucu zaten çalışıyor"));
        }

        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("-m").arg(&self.model_path)
            .arg("-c").arg(self.ctx_size.to_string())
            .arg("-t").arg(self.n_threads.to_string())
            .arg("--n-gpu-layers").arg(self.n_gpu_layers.to_string())
            .arg("--host").arg(&self.host)
            .arg("--port").arg(self.port.to_string())
            .arg("--embedding")
            .arg("--no-webui")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let mut child = cmd.spawn()
            .map_err(|e| anyhow!("llama-server başlatılamadı: {}. Binary: {:?}", e, self.binary_path))?;

        // Sağlık kontrolü - /health endpoint'ini bekle
        let _health_url = format!("http://127.0.0.1:{}/health", self.port);
        for _ in 0..60 {
            if reqwest::blocking::get(&format!("http://127.0.0.1:{}/health", self.port)).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        self.child = Some(child);
        Ok(())
    }

    /// Sunucuyu durdur
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// Embedding endpoint URL
    pub fn embed_url(&self) -> String {
        format!("http://{}:{}/v1/embeddings", self.host, self.port)
    }

    /// Sağlık kontrolü
    pub fn health_check(&self) -> bool {
        reqwest::blocking::get(&format!("http://{}:{}/health", self.host, self.port)).is_ok()
    }

    /// Sunucu çalışıyor mu?
    pub fn is_running(&self) -> bool {
        self.child.as_ref().map_or(false, |c| {
            // Simple check - try_wait would require &mut, so we just check if PID exists
            // by attempting to send signal 0 (Unix) or checking process list (Windows)
            #[cfg(not(target_os = "windows"))]
            {
                let pid = c.id();
                std::process::Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
            #[cfg(target_os = "windows")]
            {
                let pid = c.id();
                std::process::Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {}", pid)])
                    .output()
                    .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
                    .unwrap_or(false)
            }
        })
    }
}

impl Drop for LlamaServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Global singleton - uygulama boyunca tek instance
static LLAMA_SERVER: OnceLock<LlamaServer> = OnceLock::new();

/// Global llama-server instance'ını al veya oluştur
pub fn global_llama_server() -> &'static LlamaServer {
    LLAMA_SERVER.get_or_init(|| {
        // Default model path - kullanıcı ayarlayacak
        let model_path = dirs::home_dir()
            .unwrap()
            .join(".nemes")
            .join("models")
            .join("qwen2.5-3b-instruct-q4_k_m.gguf");
        
        LlamaServer::new(model_path, 1241)
            .with_gpu_layers(99)
            .with_ctx_size(2048)
            .with_threads(4)
    })
}

/// llama-server'ı indir ve `binaries/` klasörüne koy
pub fn download_llama_server() -> Result<PathBuf> {
    use std::io::Write;
    
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    std::fs::create_dir_all(&target_dir)?;

    let (url, filename) = if cfg!(target_os = "windows") {
        ("https://github.com/ggerganov/llama.cpp/releases/download/b4500/llama-b4500-bin-win64-cuda.zip", "llama-server.exe")
    } else if cfg!(target_os = "macos") {
        ("https://github.com/ggerganov/llama.cpp/releases/download/b4500/llama-b4500-bin-macos.zip", "llama-server")
    } else {
        // Linux - CUDA veya Vulkan versiyonu
        if std::path::Path::new("/usr/bin/nvidia-smi").exists() {
            ("https://github.com/ggerganov/llama.cpp/releases/download/b4500/llama-b4500-bin-ubuntu-cuda.zip", "llama-server")
        } else {
            ("https://github.com/ggerganov/llama.cpp/releases/download/b4500/llama-b4500-bin-ubuntu-vulkan.zip", "llama-server")
        }
    };

    let zip_path = target_dir.join(filename);
    
    // İndir
    println!("llama-server indiriliyor: {}", url);
    let response = blocking::get(url)?;
    let bytes = response.bytes()?;
    let mut file = std::fs::File::create(&zip_path)?;
    std::io::copy(&mut std::io::Cursor::new(&bytes), &mut file)?;
    
    // Çıkar (zip)
    let file = std::fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = target_dir.join(file.mangled_name());
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    
    // Binary'yi doğru yere taşı
    let binary_path = target_dir.join(LlamaServer::binary_name());
    std::fs::rename(target_dir.join("llama-server"), &binary_path).ok();
    std::fs::remove_file(&zip_path).ok();
    
    // Linux'ta execute permission
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))?;
    }
    
    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_llama_server_creation() {
        let server = LlamaServer::new("test.gguf", 1241);
        assert_eq!(server.port, 1241);
        assert_eq!(server.n_gpu_layers, 99);
    }
}