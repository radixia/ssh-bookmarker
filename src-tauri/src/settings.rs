use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    #[serde(default)]
    pub terminal: Option<String>,
    #[serde(default)]
    pub db_dir: Option<String>,
    #[serde(default)]
    pub hide_dock_icon: bool,
    /// When true, **Launch SSH** runs the OpenClaw maintenance one-liner
    /// over SSH and exits, instead of dropping the user into an interactive
    /// shell. See [`launcher::OPENCLAW_UPDATE_CMD`].
    #[serde(default)]
    pub update_mode: bool,
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
