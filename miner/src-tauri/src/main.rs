// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
//! NEMES Miner — Tauri ana giriş
//! Komuta sizde, güç madencide. Heartbeat olmadan kuş uçmaz (Md.113)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod heartbeat;
mod komuta;
mod models;
mod worker;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            heartbeat::get_status,
            heartbeat::start_heartbeat,
            models::list_models,
            models::pull_model,
            models::search_hf,
            models::remove_model,
            models::cancel_pull,
            models::pause_pull,
            worker::get_gpu_info,
            worker::start_mining,
            worker::stop_mining,
            komuta::kayit,
            komuta::get_komuta_status,
        ])
        .setup(|app| {
            let _window = app.get_window("main").unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("NEMES Miner çalışırken hata");
}
