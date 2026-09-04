// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
// NEMES Miner — Frontend iskelet (Ollama benzeri)
// Modeller, Chat, Mining, Dashboard sekmeleri

import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

type Tab = "dashboard" | "models" | "chat" | "mining";

let currentTab: Tab = "dashboard";
const pulling = new Set<string>();

function esc(s: string): string {
  const d = document.createElement("div");
  d.textContent = s;
  return d.innerHTML;
}

async function refreshModels() {
  const models = await invoke<any[]>("list_models");
  const el = document.getElementById("models-list")!;
  el.textContent = "";
  for (const m of models) {
    const card = document.createElement("div");
    card.className = `model-card ${m.indirilmis ? "indirilmis" : ""}`;
    const title = document.createElement("b");
    title.textContent = m.id;
    const meta = document.createElement("div");
    meta.textContent = `${m.boyut} — VRAM ${m.vram}`;
    const desc = document.createElement("small");
    desc.textContent = m.aciklama;
    card.append(title, document.createElement("br"), meta, desc, document.createElement("br"));
    if (m.indirilmis) {
      const ok = document.createElement("span");
      ok.textContent = `✅ İndirildi: ${m.yerel_yol}`;
      const btn = document.createElement("button");
      btn.textContent = "Sil";
      btn.dataset.dosya = m.dosya;
      btn.addEventListener("click", () => removeModel(m.dosya));
      card.append(ok, document.createTextNode(" "), btn);
    } else if (pulling.has(m.id)) {
      const prog = document.createElement("span");
      prog.textContent = "⏳ İndiriliyor...";
      prog.style.color = "#b8860b";
      const pauseBtn = document.createElement("button");
      pauseBtn.textContent = "Duraklat";
      pauseBtn.style.marginLeft = "8px";
      pauseBtn.addEventListener("click", () => pausePull(m.id));
      const cancelBtn = document.createElement("button");
      cancelBtn.textContent = "İptal";
      cancelBtn.style.marginLeft = "6px";
      cancelBtn.addEventListener("click", () => cancelPull(m.id));
      card.append(prog, pauseBtn, cancelBtn);
    } else {
      const btn = document.createElement("button");
      btn.textContent = "İndir";
      btn.dataset.modelId = m.id;
      btn.addEventListener("click", () => pullModel(m.id));
      card.append(btn);
    }
    el.appendChild(card);
  }
}

async function cancelPull(id: string) {
  try {
    await invoke("cancel_pull", { modelId: id });
  } catch (e: any) {
    console.log("cancel hata", e);
  }
}
async function pausePull(id: string) {
  try {
    await invoke("pause_pull", { modelId: id });
  } catch (e: any) {
    console.log("pause hata", e);
  }
}

function formatBytes(b: number) {
  if (b === 0) return "0 B";
  const u = ["B","KB","MB","GB"];
  const i = Math.floor(Math.log(b)/Math.log(1024));
  return `${(b/Math.pow(1024,i)).toFixed(1)} ${u[i]}`;
}

async function pullModel(id: string) {
  if (pulling.has(id)) return;
  pulling.add(id);
  refreshModels();
  try {
    const msg = await invoke<string>("pull_model", { modelId: id });
    console.log(msg);
    pulling.delete(id);
    refreshModels();
  } catch (e: any) {
    const errStr = String(e);
    // Timeout / duraklatıldı ise alert gösterme — progress bar zaten resume mesajı veriyor, part korundu
    if (errStr.includes("bağlantı kesildi") || errStr.includes("duraklatıldı") || errStr.includes("resume")) {
      console.log("resume edilebilir hata:", errStr);
      // 2sn sonra butonu tekrar aktif et (resume için)
      setTimeout(() => { pulling.delete(id); refreshModels(); }, 1500);
      return;
    }
    try {
      const { message } = await import("@tauri-apps/api/dialog");
      await message(errStr, { title: "NEMES Miner", type: "error" });
    } catch {
      alert(`İndirme hatası: ${errStr}`);
    }
    pulling.delete(id);
    refreshModels();
  }
}

async function removeModel(dosya: string) {
  if (!confirm(`${dosya} silinsin mi?`)) return;
  try {
    await invoke("remove_model", { dosya });
  } catch (e: any) {
    alert(`Silme hatası: ${e}`);
  }
  refreshModels();
}

async function searchHF() {
  const q = (document.getElementById("hf-search") as HTMLInputElement).value.trim();
  if (!q) return;
  const el = document.getElementById("hf-results")!;
  el.textContent = "";
  const loading = document.createElement("span");
  loading.textContent = "Aranıyor...";
  el.appendChild(loading);
  try {
    const res = await invoke<string[]>("search_hf", { query: q });
    el.textContent = "";
    if (res.length === 0) {
      const i = document.createElement("i");
      i.textContent = "Sonuç yok";
      el.appendChild(i);
    } else {
      for (const id of res) {
        const row = document.createElement("div");
        row.style.cssText = "display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid #222;";
        const span = document.createElement("span");
        span.textContent = id;
        const btn = document.createElement("button");
        btn.textContent = "İndir";
        btn.addEventListener("click", () => pullModel(id));
        row.append(span, btn);
        el.appendChild(row);
      }
    }
  } catch (e: any) {
    el.textContent = "";
    const err = document.createElement("span");
    err.style.color = "#f55";
    err.textContent = `Hata: ${e}`;
    el.appendChild(err);
  }
}

// Sağ üst ilerleme + üst bar
function setupPullProgress() {
  listen("pull-progress", (event: any) => {
    const p = event.payload as { model_id: string; indirilen: number; toplam: number; hiz: string };
    const bar = document.getElementById("pull-progress-bar") as HTMLElement;
    const notify = document.getElementById("pull-notify") as HTMLElement;
    const title = document.getElementById("pull-notify-title") as HTMLElement;
    const detail = document.getElementById("pull-notify-detail") as HTMLElement;
    const fill = document.getElementById("pull-notify-fill") as HTMLElement;
    const pct = p.toplam > 0 ? Math.round(p.indirilen / p.toplam * 100) : 0;
    bar.style.width = pct + "%";
    notify.classList.add("show");
    title.textContent = p.model_id.split("/").pop() || p.model_id;
    detail.textContent = `${formatBytes(p.indirilen)} / ${p.toplam ? formatBytes(p.toplam) : "?"} — ${pct}% — ${p.hiz}`;
    fill.style.width = pct + "%";
    if (p.hiz === "tamamlandı" || pct === 100) {
      detail.textContent += " ✅";
      pulling.delete(p.model_id);
      setTimeout(() => { notify.classList.remove("show"); bar.style.width = "0"; refreshModels(); }, 2500);
    } else if (p.hiz === "zaten indirildi") {
      pulling.delete(p.model_id);
      setTimeout(() => { notify.classList.remove("show"); bar.style.width = "0"; refreshModels(); }, 2000);
    } else if (p.hiz.includes("duraklatıldı") || p.hiz.includes("bağlantı kesildi")) {
      detail.textContent += " — tekrar İndir ile resume eder";
      pulling.delete(p.model_id);
      setTimeout(() => refreshModels(), 1000);
    } else {
      // İndirme devam ediyor — kartı yenile ki Duraklat/İptal görünsün
      if (!pulling.has(p.model_id)) {
        pulling.add(p.model_id);
        refreshModels();
      }
    }
  });
}

document.addEventListener("DOMContentLoaded", () => {
  setupPullProgress();
  // Legacy expose (not used, kept for debug)
  (window as any).searchHF = searchHF;
  (window as any).pullModel = pullModel;
  (window as any).removeModel = removeModel;

  document.getElementById("hf-search-btn")?.addEventListener("click", searchHF);
  document.getElementById("hf-search")?.addEventListener("keydown", (e) => {
    if ((e as KeyboardEvent).key === "Enter") searchHF();
  });
  document.getElementById("mining-start-btn")?.addEventListener("click", async () => {
    const log = document.getElementById("mining-log")!;
    log.textContent = "Mining başlatılıyor...";
    try {
      const r = await invoke<string>("start_mining", { komutaUrl: "http://127.0.0.1:8787", token: localStorage.getItem("nemes_token") || "test" });
      log.textContent = r;
    } catch (e: any) { log.textContent = `Hata: ${e}`; }
  });
  document.getElementById("mining-stop-btn")?.addEventListener("click", async () => {
    const log = document.getElementById("mining-log")!;
    try {
      const r = await invoke<string>("stop_mining");
      log.textContent = r;
    } catch (e: any) { log.textContent = `Hata: ${e}`; }
  });
  document.querySelectorAll("[data-tab]").forEach(btn => {
    btn.addEventListener("click", (e) => {
      const tab = (e.currentTarget as HTMLElement).dataset.tab as Tab;
      if (!tab) return;
      document.querySelectorAll(".tab").forEach(t => t.classList.remove("active"));
      document.getElementById(`tab-${tab}`)?.classList.add("active");
      if (tab === "models") refreshModels();
    });
  });
  refreshModels();

  // Komuta durumunu 5sn’de bir yokla
  setInterval(async () => {
    try {
      const s = await invoke<any>("get_komuta_status", { komutaUrl: "http://127.0.0.1:8787", token: localStorage.getItem("nemes_token") || "test" });
      const el = document.getElementById("komuta-status");
      if (el) el.textContent = s.bagli ? `Komuta: Bağlı — API hakkı ${s.api_hakki}` : "Komuta: Kilitli";
      if (el) el.style.color = s.bagli ? "#2a6" : "#f55";
    } catch {}
  }, 5000);
});
