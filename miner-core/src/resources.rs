// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub ad: String,
    pub vram_toplam: String,
    pub vram_bos: String,
    pub oneri_batch: u32,
    pub kullanim_pct: u8,
    pub sicaklik: Option<String>,
}

pub fn get_gpu_info() -> GpuInfo {
    // nvidia-smi parse — yoksa placeholder
    if let Ok(out) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total,memory.free,utilization.gpu,temperature.gpu", "--format=csv,noheader,nounits"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = s.lines().next() {
                let parts: Vec<&str> = line.split(',').map(|p| p.trim()).collect();
                if parts.len() >= 5 {
                    let vram_bos = parts[2].to_string() + " MiB";
                    let kullanim = parts[3].parse::<u8>().unwrap_or(0);
                    let sicak = format!("{}°C", parts[4]);
                    let batch = if parts[1].parse::<u32>().unwrap_or(0) > 8000 { 48 } else { 24 };
                    return GpuInfo {
                        ad: parts[0].to_string(),
                        vram_toplam: parts[1].to_string() + " MiB",
                        vram_bos,
                        oneri_batch: batch,
                        kullanim_pct: kullanim,
                        sicaklik: Some(sicak),
                    };
                }
            }
        }
    }
    // Fallback — merkez 1080 Ti varsayımı
    GpuInfo {
        ad: "GPU algılanamadı (CPU modu)".into(),
        vram_toplam: "--".into(),
        vram_bos: "--".into(),
        oneri_batch: 24,
        kullanim_pct: 0,
        sicaklik: None,
    }
}

pub fn get_cpu_info() -> String {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_cpu();
    format!("CPU {}% — {} core", sys.global_cpu_info().cpu_usage() as u8, sys.cpus().len())
}
