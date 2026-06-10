use crate::db::Bookmark;
use crate::error::{AppError, AppResult};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Remote segments composing the OpenClaw maintenance one-liner. Each is
/// joined with ` && ` and may be prefixed with `sudo ` when the bookmark
/// user isn't root (see [`build_openclaw_cmd`]).
const OPENCLAW_UPDATE_SEGMENTS: &[&str] = &[
    "apt-get update",
    "apt-get upgrade -y",
    "openclaw update",
    "openclaw doctor --fix",
];

/// The OpenClaw maintenance one-liner as it runs **when the bookmark user is
/// `root`** (no sudo prefix). Surfaced through `SettingsView.openclaw_update_cmd`
/// for the prefs UI hint.
pub const OPENCLAW_UPDATE_CMD: &str =
    "apt-get update && apt-get upgrade -y && openclaw update && openclaw doctor --fix";

/// Build the remote command, prefixing each segment with `sudo ` when the
/// SSH user is not root. Sudo's default `secure_path` on Debian/Ubuntu
/// includes `/usr/local/bin`, which fixes both the permission issue *and*
/// the non-interactive-PATH issue that hides `openclaw` from a bare ssh
/// session.
pub fn build_openclaw_cmd(user: &str) -> String {
    let prefix = if user.trim() == "root" { "" } else { "sudo " };
    OPENCLAW_UPDATE_SEGMENTS
        .iter()
        .map(|s| format!("{prefix}{s}"))
        .collect::<Vec<_>>()
        .join(" && ")
}

pub fn build_ssh_command(b: &Bookmark, update_mode: bool) -> String {
    let mut parts: Vec<String> = vec!["ssh".to_string()];

    // Force a TTY so apt-get / openclaw can print progress and prompt if needed.
    if update_mode {
        parts.push("-t".into());
    }

    if b.port != 22 {
        parts.push("-p".into());
        parts.push(b.port.to_string());
    }

    if b.auth_type == "key" {
        if let Some(path) = b.key_path.as_ref().filter(|s| !s.trim().is_empty()) {
            parts.push("-i".into());
            parts.push(shell_quote(&expand_tilde(path)));
        }
    }

    if let Some(extra) = b.extra_args.as_ref().filter(|s| !s.trim().is_empty()) {
        parts.push(extra.trim().to_string());
    }

    parts.push(format!("{}@{}", b.user, b.host));

    if update_mode {
        // Build the remote command per-bookmark (sudo prefix when not root),
        // then single-quote the whole thing for the local shell. Embedded
        // single quotes are defensively escaped.
        let remote = build_openclaw_cmd(&b.user);
        parts.push(format!("'{}'", remote.replace('\'', "'\\''")));
    }

    parts.join(" ")
}

pub fn launch(b: &Bookmark, terminal: Option<&str>, update_mode: bool) -> AppResult<String> {
    let cmd = build_ssh_command(b, update_mode);
    let label = if update_mode {
        format!("{}-openclaw-update", b.name)
    } else {
        b.name.clone()
    };
    let script_path = write_command_script(&cmd, &label)?;

    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        if let Some(bundle) = terminal.and_then(name_to_bundle) {
            command.args(["-b", bundle]);
        }
        command.arg(&script_path);
        let status = command.status()?;
        if !status.success() {
            return Err(AppError::Other(format!(
                "`open` exited with status {status}"
            )));
        }
        return Ok(cmd);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = script_path;
        let _ = terminal;
        Err(AppError::Other(
            "launching is only implemented for macOS".into(),
        ))
    }
}

#[cfg(target_os = "macos")]
fn name_to_bundle(name: &str) -> Option<&'static str> {
    match name {
        "Terminal" => Some("com.apple.terminal"),
        "iTerm" => Some("com.googlecode.iterm2"),
        "Ghostty" => Some("com.mitchellh.ghostty"),
        "Warp" => Some("dev.warp.Warp-Stable"),
        "kitty" => Some("net.kovidgoyal.kitty"),
        "Hyper" => Some("co.zeit.hyper"),
        _ => None,
    }
}

#[cfg(not(target_os = "macos"))]
fn name_to_bundle(_name: &str) -> Option<&'static str> {
    None
}

pub fn known_terminals() -> Vec<TerminalInfo> {
    let candidates: &[(&str, &[&str])] = &[
        ("Terminal", &["/System/Applications/Utilities/Terminal.app", "/Applications/Utilities/Terminal.app"]),
        ("iTerm", &["/Applications/iTerm.app"]),
        ("Ghostty", &["/Applications/Ghostty.app"]),
        ("Warp", &["/Applications/Warp.app"]),
        ("kitty", &["/Applications/kitty.app"]),
        ("Hyper", &["/Applications/Hyper.app"]),
    ];
    let home_apps = dirs::home_dir().map(|h| h.join("Applications"));
    candidates
        .iter()
        .map(|(name, paths)| {
            let installed = paths.iter().any(|p| std::path::Path::new(p).exists())
                || home_apps
                    .as_ref()
                    .map(|h| h.join(format!("{name}.app")).exists())
                    .unwrap_or(false);
            TerminalInfo {
                name: (*name).to_string(),
                installed,
            }
        })
        .collect()
}

#[derive(serde::Serialize)]
pub struct TerminalInfo {
    pub name: String,
    pub installed: bool,
}

#[cfg(target_os = "macos")]
pub fn detect_default_terminal() -> String {
    // Read LaunchServices preferences for the .command (shell-script) handler.
    let out = Command::new("defaults")
        .args([
            "read",
            "com.apple.LaunchServices/com.apple.launchservices.secure",
            "LSHandlers",
        ])
        .output();

    if let Ok(out) = out {
        if let Ok(text) = String::from_utf8(out.stdout) {
            if let Some(bundle_id) = parse_shell_script_handler(&text) {
                return bundle_to_pretty_name(&bundle_id);
            }
        }
    }
    "Terminal".to_string()
}

#[cfg(not(target_os = "macos"))]
pub fn detect_default_terminal() -> String {
    "system terminal".to_string()
}

#[cfg(target_os = "macos")]
fn parse_shell_script_handler(text: &str) -> Option<String> {
    // The plist dump is a sequence of `{ ... }` dictionaries. We look for the
    // dict that contains LSHandlerContentType = "com.apple.terminal.shell-script"
    // and pull its LSHandlerRoleAll value.
    for chunk in text.split('}') {
        if chunk.contains("com.apple.terminal.shell-script") {
            for line in chunk.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("LSHandlerRoleAll = ") {
                    let val = rest.trim_end_matches(';').trim().trim_matches('"');
                    if !val.is_empty() && val != "-" {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn bundle_to_pretty_name(bundle_id: &str) -> String {
    match bundle_id {
        "com.apple.terminal" => "Terminal".into(),
        "com.googlecode.iterm2" => "iTerm".into(),
        "com.mitchellh.ghostty" => "Ghostty".into(),
        "dev.warp.Warp-Stable" => "Warp".into(),
        "net.kovidgoyal.kitty" => "kitty".into(),
        "co.zeit.hyper" => "Hyper".into(),
        other => other.to_string(),
    }
}

fn write_command_script(ssh_cmd: &str, label: &str) -> AppResult<PathBuf> {
    let mut path = std::env::temp_dir();
    let safe_label: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    path.push(format!("ssh-bookmarker-{safe_label}-{nonce}.command"));

    // Escape single quotes in the displayed command for safe inclusion in
    // a single-quoted printf argument. We cannot use `echo '...'` directly:
    // single quotes do not nest, so any `'` inside ssh_cmd (e.g. our quoted
    // remote command) would close the echo's string and let bash interpret
    // the remainder — including any `&&` — as separate local commands.
    let echo_safe = ssh_cmd.replace('\'', "'\\''");
    let script = format!(
        "#!/bin/bash\n\
         # Auto-generated by SSH Bookmarker — safe to delete.\n\
         clear\n\
         printf '> %s\\n' '{echo_safe}'\n\
         {ssh_cmd}\n\
         status=$?\n\
         rm -- \"$0\"\n\
         exit $status\n",
        echo_safe = echo_safe,
        ssh_cmd = ssh_cmd
    );

    let mut file = std::fs::File::create(&path)?;
    file.write_all(script.as_bytes())?;
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(path)
}

fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped).to_string_lossy().into_owned();
        }
    }
    path.to_string()
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/' | '.' | ':' | '@'))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}
