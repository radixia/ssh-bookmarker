mod db;
mod error;
mod launcher;
mod settings;

use db::{Bookmark, Db};
use error::{AppError, AppResult};
use launcher::TerminalInfo;
use settings::Settings;
use std::path::PathBuf;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuEvent, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

const TRAY_ID: &str = "main";

struct AppState {
    db: Mutex<Db>,
    settings: Mutex<Settings>,
}

#[tauri::command]
async fn list_bookmarks(state: State<'_, AppState>) -> AppResult<Vec<Bookmark>> {
    state.db.lock().await.list().await
}

#[tauri::command]
async fn create_bookmark(
    app: AppHandle,
    state: State<'_, AppState>,
    bookmark: Bookmark,
) -> AppResult<Bookmark> {
    let created = state.db.lock().await.create(bookmark).await?;
    let _ = rebuild_tray_menu(&app).await;
    Ok(created)
}

#[tauri::command]
async fn update_bookmark(
    app: AppHandle,
    state: State<'_, AppState>,
    bookmark: Bookmark,
) -> AppResult<Bookmark> {
    let updated = state.db.lock().await.update(bookmark).await?;
    let _ = rebuild_tray_menu(&app).await;
    Ok(updated)
}

#[tauri::command]
async fn delete_bookmark(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<()> {
    state.db.lock().await.delete(id).await?;
    let _ = rebuild_tray_menu(&app).await;
    Ok(())
}

#[tauri::command]
async fn launch_bookmark(state: State<'_, AppState>, id: i64) -> AppResult<String> {
    let bookmark = state.db.lock().await.get(id).await?;
    let terminal = state.settings.lock().await.terminal.clone();
    launcher::launch(&bookmark, terminal.as_deref())
}

#[tauri::command]
fn default_terminal() -> String {
    launcher::detect_default_terminal()
}

#[tauri::command]
async fn check_reachable(host: String, port: u16) -> bool {
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};

    let addr = format!("{host}:{port}");
    let probe = async {
        let mut stream = TcpStream::connect(&addr).await.ok()?;
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await.ok()?;
        if &buf == b"SSH-" {
            Some(())
        } else {
            None
        }
    };
    matches!(timeout(Duration::from_millis(3500), probe).await, Ok(Some(())))
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> AppResult<SettingsView> {
    let s = state.settings.lock().await.clone();
    Ok(SettingsView::from(s))
}

#[tauri::command]
async fn set_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<SettingsView> {
    let mut current = state.settings.lock().await;
    let prev_path = settings::resolve_db_path(&current)?;
    let next_path = settings::resolve_db_path(&settings)?;
    let prev_hide = current.hide_dock_icon;

    let path_changed = next_path != prev_path;
    if path_changed {
        if let Some(parent) = next_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if !next_path.exists() && prev_path.exists() {
            std::fs::copy(&prev_path, &next_path)?;
        }
        let new_db = Db::open(next_path).await?;
        *state.db.lock().await = new_db;
    }

    *current = settings.clone();
    current.save()?;
    drop(current);

    if settings.hide_dock_icon != prev_hide {
        apply_dock_mode(&app, settings.hide_dock_icon);
    }
    let _ = path_changed;
    let _ = rebuild_tray_menu(&app).await;

    Ok(SettingsView::from(settings))
}

#[tauri::command]
fn list_terminals() -> Vec<TerminalInfo> {
    launcher::known_terminals()
}

#[tauri::command]
async fn export_bookmarks(state: State<'_, AppState>, path: String) -> AppResult<usize> {
    let items = state.db.lock().await.list().await?;
    let json = serde_json::to_vec_pretty(&items)
        .map_err(|e| AppError::Other(e.to_string()))?;
    let target = PathBuf::from(&path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&target, json)?;
    Ok(items.len())
}

#[derive(serde::Serialize)]
struct SettingsView {
    terminal: Option<String>,
    db_dir: Option<String>,
    hide_dock_icon: bool,
    effective_db_dir: String,
    default_db_dir: String,
    detected_terminal: String,
    is_macos: bool,
}

impl From<Settings> for SettingsView {
    fn from(s: Settings) -> Self {
        let default_dir = settings::default_db_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let effective = s
            .db_dir
            .clone()
            .filter(|d| !d.trim().is_empty())
            .unwrap_or_else(|| default_dir.clone());
        Self {
            terminal: s.terminal,
            db_dir: s.db_dir,
            hide_dock_icon: s.hide_dock_icon,
            effective_db_dir: effective,
            default_db_dir: default_dir,
            detected_terminal: launcher::detect_default_terminal(),
            is_macos: cfg!(target_os = "macos"),
        }
    }
}

async fn rebuild_tray_menu(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    let items = state.db.lock().await.list().await?;

    let mut builder = MenuBuilder::new(app);
    if items.is_empty() {
        let empty = MenuItemBuilder::with_id("noop", "No bookmarks yet")
            .enabled(false)
            .build(app)
            .map_err(|e| AppError::Other(e.to_string()))?;
        builder = builder.item(&empty);
    } else {
        for b in &items {
            if let Some(id) = b.id {
                let label = format!("{}  —  {}@{}", b.name, b.user, b.host);
                let item = MenuItemBuilder::with_id(format!("bm:{id}"), label)
                    .build(app)
                    .map_err(|e| AppError::Other(e.to_string()))?;
                builder = builder.item(&item);
            }
        }
    }

    let sep = PredefinedMenuItem::separator(app)
        .map_err(|e| AppError::Other(e.to_string()))?;
    let show = MenuItemBuilder::with_id("show", "Show SSH Bookmarker")
        .build(app)
        .map_err(|e| AppError::Other(e.to_string()))?;
    let quit = MenuItemBuilder::with_id("quit", "Quit")
        .build(app)
        .map_err(|e| AppError::Other(e.to_string()))?;
    builder = builder.item(&sep).item(&show).item(&quit);

    let menu = builder.build().map_err(|e| AppError::Other(e.to_string()))?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))
            .map_err(|e| AppError::Other(e.to_string()))?;
    }
    Ok(())
}

fn handle_tray_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref().to_string();
    if id == "show" {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.show();
            let _ = win.unminimize();
            let _ = win.set_focus();
        }
        return;
    }
    if id == "quit" {
        app.exit(0);
        return;
    }
    if let Some(rest) = id.strip_prefix("bm:") {
        if let Ok(bid) = rest.parse::<i64>() {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<AppState>();
                let bookmark = state.db.lock().await.get(bid).await;
                let terminal = state.settings.lock().await.terminal.clone();
                if let Ok(b) = bookmark {
                    let _ = launcher::launch(&b, terminal.as_deref());
                }
            });
        }
    }
}

fn apply_dock_mode(_app: &AppHandle, hide: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if hide {
            tauri::ActivationPolicy::Accessory
        } else {
            tauri::ActivationPolicy::Regular
        };
        let _ = _app.set_activation_policy(policy);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let settings = Settings::load();
            let db_path = settings::resolve_db_path(&settings)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            let db = tauri::async_runtime::block_on(Db::open(db_path))
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            let hide_dock = settings.hide_dock_icon;
            app.manage(AppState {
                db: Mutex::new(db),
                settings: Mutex::new(settings),
            });

            const TRAY_ICON_BYTES: &[u8] =
                include_bytes!("../icons/icon.png");
            let icon = Image::from_bytes(TRAY_ICON_BYTES)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .icon_as_template(false)
                .tooltip("SSH Bookmarker")
                .on_menu_event(handle_tray_menu_event)
                .build(app)?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = rebuild_tray_menu(&handle).await;
                apply_dock_mode(&handle, hide_dock);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_bookmarks,
            create_bookmark,
            update_bookmark,
            delete_bookmark,
            launch_bookmark,
            default_terminal,
            check_reachable,
            get_settings,
            set_settings,
            list_terminals,
            export_bookmarks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
