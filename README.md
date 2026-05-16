# SSH Bookmarker

> A native, no-friction launcher for your SSH connections — sitting quietly in your macOS menu bar.

SSH Bookmarker is a small Tauri 2 desktop app that keeps every server you care about one click away. Save the connection details once; from then on, your bookmarks live in the menu bar and open in your terminal of choice with a single click. No `~/.ssh/config` gymnastics, no shell aliases to remember, no rummaging through password managers for a hostname.

---

## Why

If you SSH into more than three or four machines, you already know the pain:

- You forget which user goes with which host.
- The port is non-default and you can never remember it.
- The right key file is buried four folders deep.
- You keep retyping `ssh -i ~/.ssh/whatever_rsa -p 2222 user@some.host.example.com` and *still* fat-finger the hostname.

SSH Bookmarker stores those parameters once, builds the correct command for you, writes a one-shot `.command` script, and hands it to whatever terminal you've set as your default — Terminal, iTerm, Ghostty, Warp, kitty, Hyper. The script self-deletes after the session ends, so there is no growing pile of junk in `/tmp`.

## Features

- **Menu-bar resident.** Every bookmark shows up as a clickable item; one click opens the session in your chosen terminal.
- **Pick your terminal.** Auto-detects the system default (via LaunchServices' `com.apple.terminal.shell-script` handler) and lets you override it. Supports Apple Terminal, iTerm2, Ghostty, Warp, kitty, and Hyper out of the box.
- **Auth flexibility.** Password (interactive) or key-based authentication; the key path supports `~` expansion.
- **Reachability probe.** Quick TCP check that verifies a host is reachable *and* actually speaks SSH (looks for the `SSH-` banner) before you bother launching.
- **Custom SSH flags.** Per-bookmark `extra_args` field for `-o`, `-L`, `-J`, etc.
- **Local-first storage.** Bookmarks live in a local SQLite database (via libSQL); no network calls, no telemetry, no cloud account.
- **Configurable storage location.** Point the DB at iCloud Drive, Dropbox, a synced folder — wherever you want your bookmarks to follow you.
- **Hide-dock mode.** Run as a pure menu-bar utility (`Accessory` activation policy) when you want it out of the Dock and ⌘-Tab switcher.
- **Export.** Dump every bookmark to a single JSON file for backup or migration.
- **Sandbox-friendly launcher.** The app never spawns `ssh` itself; it writes a short script to a temp directory and uses `open -b <bundle-id>` to hand it off, which means it works cleanly with macOS's sandbox and your terminal's own shell init.

## Screenshots

> _Pending. Drop them in `docs/` when you have them._

## Installation

### Prebuilt DMG (recommended)

A signed-or-otherwise-acquired DMG lives under `src-tauri/target/release/bundle/dmg/` after a build. Double-click it, drag **SSH Bookmarker** to `/Applications`, and launch.

> macOS may refuse to open the unsigned app on first run. Right-click the app and choose **Open** to bypass Gatekeeper for that one launch.

### From source

```bash
git clone https://github.com/aletheia/ssh-bookmarker.git
cd ssh-bookmarker
npm install
npm run tauri dev     # development build with hot-reload
npm run tauri build   # production .app + .dmg
```

Requirements:
- macOS 11+ (Apple silicon or Intel)
- Node.js 20+
- Rust toolchain (`rustup`, stable)
- Xcode Command Line Tools

## Usage

### Adding a bookmark

1. Launch the app. The main window lists every saved bookmark.
2. Click **New bookmark**. Fill in:
   - **Name** — anything human-readable; the menu bar uses this label.
   - **Host** — DNS name or IP.
   - **User** — the SSH username.
   - **Port** — defaults to 22.
   - **Auth type** — `password` (interactive prompt by SSH) or `key` (pick a private key file).
   - **Key path** — only when auth is `key`. `~` expands to your home directory.
   - **Extra args** — any flags you want appended verbatim (e.g. `-o ServerAliveInterval=30 -L 5432:db.internal:5432`).
   - **Notes** — free text for your own reference.
3. Save. The bookmark immediately appears in the menu-bar dropdown.

### Launching

- From the **main window**: select a bookmark, click **Connect**.
- From the **menu bar**: click the SSH Bookmarker icon, then the bookmark name.

In both cases, SSH Bookmarker:
1. Builds the `ssh` command from the saved fields.
2. Writes a one-shot `.command` script to your temp directory.
3. Tells macOS to `open` that script with your selected terminal app.
4. The script clears the terminal, prints the command (so you can see exactly what was run), executes it, and deletes itself.

### Settings

Open **Settings** from the main window. You can change:

- **Default terminal** — overrides the system default for new launches.
- **Database location** — point at any directory. When you change it, the existing DB is copied to the new location if the target doesn't already have one. Existing bookmarks come with you.
- **Hide dock icon** — switches the app to macOS `Accessory` activation policy (no Dock tile, no ⌘-Tab entry). The menu-bar icon stays visible.

Settings are persisted at `~/Library/Application Support/ssh-bookmarker/settings.json`.

### Exporting

From the main window, **Export** dumps every bookmark to a JSON file. The schema matches the internal `Bookmark` struct (see [`src-tauri/src/db.rs`](src-tauri/src/db.rs)), so you can round-trip into other tools or your own scripts.

## Storage layout

```
~/Library/Application Support/ssh-bookmarker/
├── bookmarks.db        # libSQL/SQLite database (default location)
└── settings.json       # user preferences
```

If you set a custom **Database location** in Settings, only `bookmarks.db` moves; `settings.json` stays in the default app-support directory.

The DB schema is intentionally tiny:

```sql
CREATE TABLE bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    user TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 22,
    auth_type TEXT NOT NULL CHECK(auth_type IN ('password','key')),
    key_path TEXT,
    extra_args TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Nothing is encrypted at rest. **Do not store passwords here.** Use key-based auth and keep the key files where they belong (with proper file permissions).

## Architecture

```
┌──────────────────────┐       Tauri IPC        ┌──────────────────────┐
│  Frontend (TS/Vite)  │ ◄─────────────────────► │  Backend (Rust)      │
│  src/main.ts         │   invoke(...) commands  │  src-tauri/src/*.rs  │
│  src/styles.css      │                         │                      │
│  index.html          │                         │  • db.rs   (libSQL)  │
└──────────────────────┘                         │  • launcher.rs       │
                                                 │  • settings.rs       │
                                                 │  • lib.rs (commands  │
                                                 │      + tray menu)    │
                                                 └──────────────────────┘
```

- **Frontend** is plain TypeScript + Vite, no framework. The DOM is small enough that React would be overkill.
- **Backend** is Rust. Async runtime is `tokio`. DB access goes through `libsql` (a SQLite-compatible embedded driver). The tray menu is built with Tauri 2's `tray` API and rebuilt on every bookmark mutation.
- **IPC** is Tauri's standard `invoke` over the command bridge.

### Tauri commands (the API surface)

| Command            | Description                                                                 |
|--------------------|-----------------------------------------------------------------------------|
| `list_bookmarks`   | Returns every saved bookmark, alphabetised.                                 |
| `create_bookmark`  | Insert a bookmark; rebuilds the tray menu on success.                       |
| `update_bookmark`  | Update by id; rebuilds the tray menu.                                       |
| `delete_bookmark`  | Delete by id; rebuilds the tray menu.                                       |
| `launch_bookmark`  | Build the ssh command and open it in the configured terminal.               |
| `default_terminal` | Detect the system default terminal via LaunchServices.                      |
| `list_terminals`   | List known terminals and whether each is installed.                         |
| `check_reachable`  | TCP probe with SSH banner check (3.5s timeout).                             |
| `get_settings`     | Return the current settings + derived fields (effective db dir, etc.).      |
| `set_settings`     | Persist settings; copy DB to new location if changed; flip activation mode. |
| `export_bookmarks` | Write every bookmark to a JSON file at the given path.                      |

All command implementations live in [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs).

## Building a release

```bash
npm run tauri build
```

Outputs:
- **App bundle:** `src-tauri/target/release/bundle/macos/SSH Bookmarker.app`
- **DMG installer:** `src-tauri/target/release/bundle/dmg/SSH Bookmarker_<version>_<arch>.dmg`

Bump `version` in both [`package.json`](package.json) and [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) before tagging a release; they must stay in sync.

## Troubleshooting

**The menu-bar icon doesn't appear.**
Make sure the tray icon image is non-empty. The app builds with the bundled `icons/icon.png`; if you customise this, verify the replacement actually renders. The tray is configured to always be visible regardless of the "Hide dock icon" preference.

**The default terminal detection is wrong.**
SSH Bookmarker reads `defaults read com.apple.LaunchServices/com.apple.launchservices.secure LSHandlers` and looks for a handler for `com.apple.terminal.shell-script`. If you've never explicitly set one (right-click any `.command` file → **Open With** → **Always Open With...**), macOS may report none. Set one, or just override it in Settings.

**A connection opens in the wrong app.**
The launcher only uses your selected terminal when SSH Bookmarker recognises it. Unknown terminals fall back to the system default. Check `src-tauri/src/launcher.rs`'s `name_to_bundle` for the supported list.

**`open` fails or the script doesn't execute.**
Verify your temp directory is writable, and that the chosen terminal has been granted permission to run shell scripts (System Settings → Privacy & Security → Automation).

## Platform support

Currently macOS only. The launcher uses `open -b <bundle-id>` which is mac-specific; the rest of the codebase is portable. Contributions to add Linux (`xdg-open` + a terminal-emulator detection heuristic) and Windows (Windows Terminal profiles) are welcome.

## License

Not yet specified. Treat as "all rights reserved" until a `LICENSE` file is added.

## Acknowledgements

- [Tauri](https://tauri.app) — the framework that makes a small Rust+webview app this pleasant.
- [libSQL](https://github.com/tursodatabase/libsql) — embedded SQLite-compatible store with a sane async API.
- Every terminal app this thing launches into. You all have my respect.
