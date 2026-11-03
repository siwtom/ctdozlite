//! Cross-platform sandbox data directory & Multi-Account profile path resolver.
//!
//! - On Desktop (Windows / Linux / macOS): defaults to current working directory (`.`) for backward
//!   compatibility with existing `session.json`, `threads_cache.json`, and `zalo_history.db`, or
//!   `ZALO_DATA_DIR` when specified.
//! - On Mobile (iOS / Android): resolves to the app's writable sandbox container
//!   (`NSHomeDirectory()/Documents` on iOS, `internal_data_path()` on Android) so file I/O never
//!   hits `Read-Only File System` (`PermissionDenied`) errors.
//! - For Multi-Account (`slot_id >= 1`): isolates each account's credentials, conversation cache,
//!   and SQLite history under `profiles/account_<slot_id>/` while sharing `stickers_cache/`.

use std::path::PathBuf;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

static CUSTOM_DATA_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Explicitly configure the writable application data directory (used by Android `android_main`
/// and iOS `start_zalo_ios_app` entry points before launching `eframe`).
pub fn set_app_data_dir(dir: impl Into<PathBuf>) {
    let p = dir.into();
    let _ = std::fs::create_dir_all(&p);
    if let Ok(mut guard) = CUSTOM_DATA_DIR.write() {
        *guard = Some(p);
    }
}

/// Resolve the base writable directory for application state and caches.
pub fn app_data_dir() -> PathBuf {
    if let Ok(guard) = CUSTOM_DATA_DIR.read() {
        if let Some(ref p) = *guard {
            return p.clone();
        }
    }
    if let Ok(env_dir) = std::env::var("ZALO_DATA_DIR") {
        let trimmed = env_dir.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            let _ = std::fs::create_dir_all(&p);
            return p;
        }
    }
    #[cfg(target_os = "ios")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join("Documents").join("ctdozlite");
            let _ = std::fs::create_dir_all(&p);
            return p;
        }
    }
    #[cfg(target_os = "android")]
    {
        let fallback = PathBuf::from("/data/user/0/net.ctdo.ctdozlite/files");
        let _ = std::fs::create_dir_all(&fallback);
        return fallback;
    }
    PathBuf::from(".")
}

/// Directory for a specific account slot (`0` = root app data dir, `>= 1` = `profiles/account_{slot}`).
pub fn profile_dir_for_slot(slot: usize) -> PathBuf {
    let base = app_data_dir();
    if slot == 0 {
        base
    } else {
        let dir = base.join("profiles").join(format!("account_{slot}"));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }
}

pub fn session_file_for_slot(slot: usize) -> String {
    profile_dir_for_slot(slot)
        .join("session.json")
        .to_string_lossy()
        .into_owned()
}

pub fn threads_cache_for_slot(slot: usize) -> String {
    profile_dir_for_slot(slot)
        .join("threads_cache.json")
        .to_string_lossy()
        .into_owned()
}

pub fn history_db_for_slot(slot: usize) -> String {
    profile_dir_for_slot(slot)
        .join("zalo_history.db")
        .to_string_lossy()
        .into_owned()
}

pub fn app_settings_file() -> PathBuf {
    app_data_dir().join("app_settings.json")
}

pub fn stickers_cache_dir() -> PathBuf {
    let d = app_data_dir().join("stickers_cache");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn accounts_index_file() -> PathBuf {
    app_data_dir().join("accounts_index.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsIndex {
    pub slots: Vec<usize>,
    pub active_slot: usize,
}

impl Default for AccountsIndex {
    fn default() -> Self {
        Self {
            slots: vec![0],
            active_slot: 0,
        }
    }
}

pub fn load_accounts_index() -> AccountsIndex {
    let path = accounts_index_file();
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(mut idx) = serde_json::from_str::<AccountsIndex>(&raw) {
            if !idx.slots.contains(&0) {
                idx.slots.insert(0, 0);
            }
            idx.slots.sort_unstable();
            idx.slots.dedup();
            if !idx.slots.contains(&idx.active_slot) {
                idx.active_slot = 0;
            }
            return idx;
        }
    }
    // Auto-discover any existing `profiles/account_*` directories on disk
    let mut slots = vec![0usize];
    let profiles_root = app_data_dir().join("profiles");
    if let Ok(entries) = std::fs::read_dir(profiles_root) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(num_s) = name.strip_prefix("account_") {
                    if let Ok(slot_n) = num_s.parse::<usize>() {
                        if slot_n > 0 && entry.path().join("session.json").exists() {
                            slots.push(slot_n);
                        }
                    }
                }
            }
        }
    }
    slots.sort_unstable();
    slots.dedup();
    AccountsIndex {
        slots,
        active_slot: 0,
    }
}

pub fn save_accounts_index(idx: &AccountsIndex) {
    if let Ok(json) = serde_json::to_string_pretty(idx) {
        let _ = std::fs::write(accounts_index_file(), json);
    }
}
