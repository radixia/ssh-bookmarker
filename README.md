<p align="center">
  <img src="docs/cover.png" alt="SSH Bookmarker" width="100%" />
</p>

# SSH Bookmarker

> A native, no-friction launcher for your SSH connections — living quietly in your macOS menu bar.

<p align="center">
  <a href="#platform-support"><img alt="Platform: macOS" src="https://img.shields.io/badge/platform-macOS%2011%2B-black?logo=apple&logoColor=white"></a>
  <a href="https://tauri.app"><img alt="Built with Tauri 2" src="https://img.shields.io/badge/built%20with-Tauri%202-FFC131?logo=tauri&logoColor=black"></a>
  <a href="#architecture"><img alt="Rust + TypeScript" src="https://img.shields.io/badge/stack-Rust%20%2B%20TypeScript-blue?logo=rust&logoColor=white"></a>
  <a href="https://www.sqlite.org"><img alt="Storage: libSQL / SQLite" src="https://img.shields.io/badge/storage-libSQL%20%2F%20SQLite-003B57?logo=sqlite&logoColor=white"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-green.svg"></a>
  <a href="CONTRIBUTING.md"><img alt="PRs welcome" src="https://img.shields.io/badge/PRs-welcome-brightgreen"></a>
  <a href="https://github.com/aletheia/ssh-bookmarker/releases"><img alt="Version" src="https://img.shields.io/badge/version-0.3.0-blue"></a>
  <br>
  <a href="https://github.com/aletheia/ssh-bookmarker/stargazers"><img alt="GitHub stars" src="https://img.shields.io/github/stars/aletheia/ssh-bookmarker?style=social"></a>
  <a href="https://github.com/aletheia/ssh-bookmarker/issues"><img alt="GitHub issues" src="https://img.shields.io/github/issues/aletheia/ssh-bookmarker"></a>
  <a href="https://github.com/aletheia/ssh-bookmarker/commits/main"><img alt="Last commit" src="https://img.shields.io/github/last-commit/aletheia/ssh-bookmarker"></a>
</p>

SSH Bookmarker is a small Tauri 2 desktop app that keeps every server you care about one click away. Save the connection details once; from then on, your bookmarks live in the menu bar and open in your terminal of choice with a single click. No `~/.ssh/config` gymnastics, no shell aliases to remember, no rummaging through password managers for a hostname.

---

## Table of contents

- [Why](#why)
- [Quickstart](#quickstart)
- [Features](#features)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Usage](#usage)
- [Settings](#settings)
- [Storage layout](#storage-layout)
- [Architecture](#architecture)
- [Building a release](#building-a-release)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)
- [Roadmap](#roadmap)
- [Platform support](#platform-support)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## Why

If you SSH into more than three or four machines, you already know the pain:

- You forget which user goes with which host.
- The port is non-default and you can never remember it.
- The right key file is buried four folders deep.
- You keep retyping `ssh -i ~/.ssh/whatever_rsa -p 2222 user@some.host.example.com` and *still* fat-finger the hostname.

There are other ways to manage this, and they're all fine — until they're not:

| Approach              | Strength                          | Pain                                                                 |
|-----------------------|-----------------------------------|----------------------------------------------------------------------|
| `~/.ssh/config`       | Universal, scriptable             | Lives in a text file you have to open; no GUI; doesn't sync visually |
| Shell aliases / funcs | Fast typing                       | One more thing to remember; doesn't show "is this host reachable?"   |
| iTerm profiles        | Native to one terminal            | Locked to iTerm; no menu-bar quick-launch                            |
| Password managers     | Great for credentials             | Wrong tool for "open a shell to host X"                              |
| **SSH Bookmarker**    | One click, any terminal, menu bar | macOS-only (today); no on-screen password storage                    |

SSH Bookmarker stores connection parameters once, builds the correct command for you, writes a one-shot `.command` script, and hands it to whatever terminal you've set as your default — Terminal, iTerm, Ghostty, Warp, kitty, or Hyper. The script self-deletes after the session ends, so there is no growing pile of junk in `/tmp`.

## Quickstart

```bash
# 1. Get the app
git clone https://github.com/aletheia/ssh-bookmarker.git
cd ssh-bookmarker
npm install
npm run tauri build

# 2. Drag the bundle to /Applications
open src-tauri/target/release/bundle/dmg/

# 3. Launch it from /Applications (right-click → Open the first time)
#    Add a bookmark, click Launch SSH.
```

Prefer not to build? See [Installation → Prebuilt DMG](#prebuilt-dmg) for the prebuilt download path.

## Features

- **Menu-bar resident.** Every bookmark shows up as a clickable item; one click opens the session in your chosen terminal.
- **Reachability LED.** Each bookmark card shows a live status indicator (green / amber / red) based on a quick TCP probe that also confirms the host actually speaks SSH (looks for the `SSH-` banner). No more clicking only to find the box is offline.
- **Pick your terminal.** Auto-detects the system default (via LaunchServices' `com.apple.terminal.shell-script` handler) and lets you override it. Ships with native support for Apple Terminal, iTerm2, Ghostty, Warp, kitty, and Hyper.
- **Auth flexibility.** Password (interactive) or key-based authentication; the key path supports `~` expansion.
- **Custom SSH flags.** Per-bookmark `extra_args` field for `-o`, `-L`, `-J`, `-A`, anything `ssh` accepts.
- **Edit / Duplicate / Delete.** Every card has a kebab menu for in-place edits without leaving the list.
- **Local-first storage.** Bookmarks live in a local SQLite database (via libSQL). No network calls, no telemetry, no cloud account.
- **Configurable storage location.** Point the DB at iCloud Drive, Dropbox, a synced folder — wherever you want your bookmarks to follow you.
- **Menu-bar-only mode.** Hide the Dock icon to run as a pure menu-bar utility (`Accessory` activation policy). The menu-bar icon stays visible.
- **Monochrome tray icon.** A proper macOS template image — tints itself black or white to match light/dark menu bars.
- **JSON export.** Dump every bookmark to a single file for backup, migration, or scripting against.
- **Update mode (OpenClaw).** When toggled on, **Launch SSH** runs a remote maintenance one-liner (`apt-get update && apt-get upgrade -y && openclaw update && openclaw doctor --fix`) over SSH and exits, instead of dropping into an interactive shell. Useful for one-click fleet maintenance across every bookmark.
- **Sandbox-friendly launcher.** The app never spawns `ssh` itself; it writes a short script to a temp directory and uses `open -b <bundle-id>` to hand it off. That works cleanly with macOS's sandbox and your terminal's own shell init (login shell, dotfiles, the works).

## Screenshots

> _Coming soon._ Drop them in `docs/` and reference them here.
>
> Suggested set: main list view with reachability LEDs, menu-bar dropdown, settings dialog, the in-place edit form.

## Installation

### Prebuilt DMG

A built DMG lives under `src-tauri/target/release/bundle/dmg/` after a local build. Once a tagged release is available on GitHub, you'll be able to grab it from the [Releases](https://github.com/aletheia/ssh-bookmarker/releases) page instead.

To install:

1. Open the DMG.
2. Drag **SSH Bookmarker** to `/Applications`.
3. Launch. On first run, macOS may refuse to open the unsigned app — right-click → **Open** to bypass Gatekeeper for that one launch.

### From source

```bash
git clone https://github.com/aletheia/ssh-bookmarker.git
cd ssh-bookmarker
npm install
npm run tauri dev     # development build with hot-reload
npm run tauri build   # production .app + .dmg
```

**Requirements**

- macOS 11+ (Apple silicon or Intel)
- Node.js 20+
- Rust toolchain (`rustup`, stable channel)
- Xcode Command Line Tools (`xcode-select --install`)

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
   - **Extra args** — flags appended verbatim (e.g. `-o ServerAliveInterval=30 -L 5432:db.internal:5432`).
   - **Notes** — free text for your own reference.
3. Save. The bookmark immediately appears in the menu-bar dropdown.

Each card shows a small **LED** next to the name: green if the host accepts SSH, red if unreachable, amber while checking.

### Launching

- From the **main window**: click **Launch SSH** on any card.
- From the **menu bar**: click the SSH Bookmarker icon, then the bookmark name.

Under the hood, both paths run the same sequence:

```
You click "Launch SSH"
        │
        ▼
┌────────────────────────────────┐
│ 1. Build `ssh …` command       │   from the saved fields
│ 2. Write self-deleting script  │   /tmp/ssh-bookmarker-<name>-<nonce>.command
│ 3. `open -b <bundle-id>` it    │   bundle-id from terminal preference
│ 4. Terminal opens, runs script │   echoes the command, runs ssh,
│ 5. Script `rm "$0"` on exit    │   leaves no trace
└────────────────────────────────┘
```

The script clears the terminal, prints the exact command (so you can see what was run), executes it, and removes itself when the session ends.

### Editing and duplicating

Each bookmark card has a kebab menu (`⋮`) with **Edit**, **Duplicate**, and **Delete**. Duplicate is handy when you have a family of similar hosts that differ only by name/IP.

### Keyboard shortcuts

- **Esc** — close the Settings dialog or any open card menu.

(More shortcuts are on the roadmap; PRs welcome.)

## Settings

Open **Settings** from the main window. You can change:

- **Default terminal** — overrides the system default for new launches. The select shows whether each known terminal is installed.
- **Database location** — point at any directory. When you change it, the existing DB is copied to the new location if the target doesn't already have one. Your bookmarks come with you. Use **Reset to default** to move back to `~/Library/Application Support/ssh-bookmarker`.
- **Hide dock icon** — switches the app to macOS `Accessory` activation policy (no Dock tile, no ⌘-Tab entry). The menu-bar icon stays visible.
- **Update mode** — when enabled, every **Launch SSH** click runs the OpenClaw maintenance command instead of opening a shell:
  ```sh
  apt-get update && apt-get upgrade -y && openclaw update && openclaw doctor --fix
  ```
  The connection exits when the command finishes. Intended for hosts you reach as root or with passwordless sudo; if you need sudo, prefix it via the bookmark's **Extra args**.
- **Export bookmarks** — writes every bookmark to a JSON file.

Settings are persisted at `~/Library/Application Support/ssh-bookmarker/settings.json`.

## Storage layout

```
~/Library/Application Support/ssh-bookmarker/
├── bookmarks.db        # libSQL/SQLite database (default location)
└── settings.json       # user preferences
```

If you set a custom **Database location** in Settings, only `bookmarks.db` moves; `settings.json` stays in the default app-support directory (it has to live somewhere fixed for the app to find on startup).

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

Nothing is encrypted at rest. **Do not store passwords here** — the schema has no field for them and we will not add one. Use key-based auth and protect the key files with proper permissions.

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

- **Frontend** is plain TypeScript + Vite. No framework — the DOM is small enough that React would be overkill.
- **Backend** is Rust. Async runtime is `tokio`. DB access goes through `libsql` (a SQLite-compatible embedded driver). The tray menu is built with Tauri 2's `tray` API and rebuilt on every bookmark mutation, so it never goes stale.
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
| `check_reachable`  | TCP probe with SSH banner check (3.5 s timeout). Powers the per-card LED.   |
| `get_settings`     | Return the current settings + derived fields (effective db dir, etc.).      |
| `set_settings`     | Persist settings; copy DB to new location if changed; flip activation mode. |
| `export_bookmarks` | Write every bookmark to a JSON file at the given path.                      |

All command implementations live in [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs). The non-obvious invariants the code relies on are documented in [CLAUDE.md](CLAUDE.md) — read that before making structural changes.

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
The tray bundles `src-tauri/icons/tray-icon.png` as a *template image* (`icon_as_template(true)`), which means macOS uses only the alpha channel. If you've replaced the icon and it's blank, the menu bar will render nothing. Regenerate with `python3 scripts/gen-tray-icon.py` or supply your own grayscale+alpha PNG with non-empty alpha. The tray is configured to **always** be visible regardless of the "Hide dock icon" preference — that's intentional.

**The default terminal detection is wrong.**
SSH Bookmarker reads `defaults read com.apple.LaunchServices/com.apple.launchservices.secure LSHandlers` and looks for a handler for `com.apple.terminal.shell-script`. If you've never explicitly set one (right-click any `.command` file → **Open With** → **Always Open With…**), macOS may report none. Set one, or just override it in Settings.

**A connection opens in the wrong app.**
The launcher only routes to your selected terminal when SSH Bookmarker recognises it. Unknown terminals fall back to the system default. The supported list lives in [`src-tauri/src/launcher.rs`](src-tauri/src/launcher.rs) (`name_to_bundle`); adding a new one takes ~5 lines.

**`open` fails or the script doesn't execute.**
Verify your temp directory is writable, and that the chosen terminal has been granted permission to run shell scripts (System Settings → Privacy & Security → Automation → enable SSH Bookmarker → your terminal).

**The reachability LED is always red even though I can SSH manually.**
The probe is a 3.5-second TCP connect plus a check for the `SSH-` banner. Firewalls that drop unsolicited probes, port-knocking setups, or hosts behind a jump box will all show as red. The LED is a convenience signal, not a precondition — clicking **Launch SSH** still works.

## FAQ

**Does it store my passwords?**
No. The schema doesn't have a password field, and we won't add one. Use key-based auth; let `ssh` prompt for passphrases when needed.

**Can I sync my bookmarks across machines?**
Yes — point the **Database location** at a synced folder (iCloud Drive, Dropbox, Syncthing, etc.). The DB file is a regular SQLite database.

**Does it work with jump hosts / ProxyJump?**
Yes. Put `-J user@jump.example.com` in the **Extra args** field. Anything `ssh` accepts as a flag goes there.

**Will there be a Linux/Windows version?**
Not today. The launcher path is macOS-specific (`open -b <bundle-id>`); everything else is portable. Contributions welcome — see [Platform support](#platform-support).

**Why a desktop app instead of a CLI tool?**
The menu-bar dropdown is the killer feature. A CLI could store the same data, but it wouldn't give you a clickable list of "every host I care about" within one keystroke of anywhere on the system.

**Why libSQL instead of plain `rusqlite`?**
`libsql` exposes a clean async API (no blocking calls inside Tauri commands) and the wire-compatible SQLite format means the file is fully portable. There is currently no remote/replicated mode being used.

## Roadmap

Not promises — directions:

- [ ] Tagged GitHub releases with a signed, notarised DMG.
- [ ] Search / filter the bookmark list (and the menu-bar dropdown).
- [ ] Keyboard shortcut to focus the search box.
- [ ] Grouping / tags for bookmarks.
- [ ] Linux support (`xdg-open` + terminal-emulator heuristic).
- [ ] Windows support (Windows Terminal profiles).
- [ ] Import from `~/.ssh/config`.
- [ ] Built-in tunnel manager (visualise active `-L` / `-R` sessions).

If any of these matter to you, open an issue and we'll prioritise.

## Platform support

Currently **macOS only**. The launcher uses `open -b <bundle-id>` which is mac-specific; the rest of the codebase is portable. The known-blockers for porting:

- **Linux** — need an alternative to `open -b` (probably `gtk-launch` or terminal-specific invocations), and a `detect_default_terminal()` replacement (`xdg-mime query default` against `application/x-shellscript`).
- **Windows** — Windows Terminal profiles are the natural model; the script-writing approach can be adapted with `.bat`/`.ps1`.

Contributions for both are welcome. Please open an issue first to align on the abstraction before coding.

## Contributing

PRs and issues are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, conventions, and the (short) list of things that have bitten contributors before. If you're an AI agent (or a human reasoning structurally about the codebase), [CLAUDE.md](CLAUDE.md) documents the invariants that aren't obvious from the code.

## License

[MIT](LICENSE) © aletheia.

## Acknowledgements

- [Tauri](https://tauri.app) — the framework that makes a small Rust+webview app this pleasant.
- [libSQL](https://github.com/tursodatabase/libsql) — embedded SQLite-compatible store with a sane async API.
- [Vite](https://vitejs.dev) — fast frontend tooling, even for a no-framework app.
- Every terminal app this thing launches into. You all have my respect.
