use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default)]
    pub terminal: Option<String>,
    #[serde(default)]
    pub db_dir: Option<String>,
    #[serde(default)]
    pub hide_dock_icon: bool,
    /// Master switch for OpenClaw features. When true, bookmark cards expose
    /// extra actions in the kebab menu (e.g. **Run OpenClaw update**).
    /// **Launch SSH** itself is unaffected — it always opens an interactive
    /// shell.
    #[serde(default, alias = "update_mode")]
    pub openclaw_enabled: bool,
    /// When true (default), the SSH session closes once the OpenClaw
    /// maintenance command finishes. When false, an interactive login
    /// shell is `exec`-ed on the remote so the user lands in a normal
    /// shell after the command completes — useful for inspecting output
    /// or doing follow-up work.
    #[serde(default = "default_true")]
    pub openclaw_exit_on_finish: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            terminal: None,
            db_dir: None,
            hide_dock_icon: false,
            openclaw_enabled: false,
            openclaw_exit_on_finish: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = match settings_path() {
            Ok(p) => p,
            Err(_) => return Self::default(),
        };
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> AppResult<()> {
        let path = settings_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| AppError::Other(e.to_string()))?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

pub fn settings_path() -> AppResult<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| AppError::Other("could not resolve data dir".into()))?;
    Ok(base.join("ssh-bookmarker").join("settings.json"))
}

pub fn default_db_dir() -> AppResult<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| AppError::Other("could not resolve data dir".into()))?;
    Ok(base.join("ssh-bookmarker"))
}

pub fn resolve_db_path(s: &Settings) -> AppResult<PathBuf> {
    let dir = match s.db_dir.as_ref() {
        Some(d) if !d.trim().is_empty() => PathBuf::from(d),
        _ => default_db_dir()?,
    };
    Ok(dir.join("bookmarks.db"))
}
