# CLAUDE.md

Working notes for AI coding agents (and humans) operating on this repo. Read this before touching code.

## What this app is

SSH Bookmarker is a macOS-only Tauri 2 desktop app that stores SSH connection details and launches them in the user's preferred terminal. It runs as a regular windowed app or as a pure menu-bar utility, depending on the `hide_dock_icon` preference. Bookmarks live in a local libSQL/SQLite database.

The full user-facing story is in [README.md](README.md). This file focuses on the things you need to know to modify the code without breaking it.

## Stack

- **Frontend:** TypeScript + Vite, no framework. Single entry `src/main.ts`. CSS is hand-rolled in `src/styles.css`. The frontend talks to the backend exclusively via `@tauri-apps/api/core` `invoke(...)` calls.
- **Backend:** Rust, edition 2021. Async runtime is `tokio` (multi-thread). DB driver is `libsql` (SQLite-compatible). The Tauri runtime version is 2.x.
- **IPC:** Tauri commands defined with `#[tauri::command]` in `src-tauri/src/lib.rs` and wired into `tauri::generate_handler!(...)` at the bottom of `run()`.
- **Build:** `npm run tauri build`. Frontend is built into `dist/`, Rust binary into `src-tauri/target/release/`, and bundles into `src-tauri/target/release/bundle/{macos,dmg}/`.

## Repo layout

```
ssh-bookmarker/
├── index.html              # Vite entry; contains the entire DOM skeleton
├── package.json            # frontend deps + npm scripts (dev / build / tauri)
├── tsconfig.json
├── vite.config.ts
├── src/
│   ├── main.ts             # all frontend logic (form, list, settings dialog)
│   └── styles.css
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json     # productName, version, bundle config
│   ├── build.rs            # tauri-build invocation
│   ├── capabilities/
│   │   └── default.json    # permissions: core / opener / dialog defaults
│   ├── icons/              # app icons + tray icon
│   └── src/
│       ├── main.rs         # one-liner; defers to lib::run()
│       ├── lib.rs          # ★ tauri commands, tray menu, dock mode, setup()
│       ├── db.rs           # libsql wrapper + Bookmark struct + migration
│       ├── launcher.rs     # ssh command construction, terminal detection
│       ├── settings.rs     # Settings struct + JSON persistence
│       └── error.rs        # AppError / AppResult
└── scripts/
    └── gen-tray-icon.py    # generates the monochrome '>_' template icon
```

`src-tauri/gen/` and `src-tauri/target/` are build artefacts; do not commit. Both are covered by `.gitignore`.

## Key invariants

- **Backend version of `version` and `tauri.conf.json` must match `package.json`.** Bump both when releasing.
- **Tray menu must be rebuilt after every mutating bookmark command.** That is currently done by calling `rebuild_tray_menu(&app).await` at the end of `create_bookmark`, `update_bookmark`, `delete_bookmark`, and `set_settings`. If you add new mutating commands, do the same.
- **The tray must always be visible.** Earlier code called `tray.set_visible(hide_dock_icon)`, which silently hid the icon when the user didn't enable hide-dock mode (the default). The current `apply_dock_mode` only touches the activation policy. Do not re-introduce conditional tray visibility.
- **`set_settings` handles a DB-path change.** If `db_dir` changes, it creates the new parent directory, copies the existing DB if the destination has none, opens a new `Db`, and swaps it into `AppState`. Preserve that order — if you open the new DB before the copy, you'll create an empty schema and orphan the old data.
- **`launcher::launch` writes a self-deleting `.command` script to the temp dir.** It never spawns `ssh` directly. This keeps macOS sandbox + Automation permissions clean. Do not "simplify" by calling `Command::new("ssh")`.
- **`launcher::build_ssh_command` takes an `update_mode: bool`.** When true, the SSH command forces a TTY (`-t`), appends `'<OPENCLAW_UPDATE_CMD>'` as a single shell-quoted argument, and the connection exits when the remote command returns. `OPENCLAW_UPDATE_CMD` is a `pub const` so it's also surfaced through `SettingsView.openclaw_update_cmd` to the frontend (the prefs modal renders it verbatim).
- **`Settings.openclaw_enabled` is a master switch, NOT a launch modifier.** `launch_bookmark` and the tray-menu launch path **always** call `launcher::launch(.., false)` — interactive shell, never the maintenance command. The OpenClaw command runs only via the separate `run_openclaw_update` Tauri command, which is invoked from the per-card kebab menu and refuses to run when `openclaw_enabled` is false. The setting field has `#[serde(alias = "update_mode")]` so older settings files migrate without losing state. If you add more OpenClaw actions, gate them on `openclaw_enabled` the same way.
- **`auth_type` is constrained to `'password'` or `'key'`** at the DB level (`CHECK` constraint). If you add a new auth mode, update the migration and the frontend form simultaneously.
- **Tauri commands return `AppResult<T>`**, not `Result<T, String>`. `AppError` implements `Serialize` via the `serde` derive on `Other(String)` etc.; keep that path so the frontend sees structured errors.

## Common operations

### Run in dev

```bash
npm run tauri dev
```

Hot-reloads the frontend; the Rust side recompiles on save. The dev URL is `http://localhost:1420`.

### Add a new Tauri command

1. Write the `#[tauri::command]` fn in `src-tauri/src/lib.rs` (or a submodule re-exported there).
2. Add it to the `tauri::generate_handler!(...)` list at the bottom of `run()`. **Easy to forget — the command will silently 404 from the frontend if you miss this.**
3. Call from TS with `invoke<ReturnType>("snake_case_name", { ... })`.
4. If it mutates bookmarks, call `rebuild_tray_menu(&app).await` before returning.

### Add a supported terminal

Two places in `src-tauri/src/launcher.rs`:
1. `name_to_bundle()` — map the display name to its bundle ID.
2. `known_terminals()` — add the install-path candidates.
3. `bundle_to_pretty_name()` — for `detect_default_terminal()` round-tripping.

The frontend pulls the list dynamically via `list_terminals`, so no UI changes are needed.

### Change the DB schema

Edit `migrate()` in `src-tauri/src/db.rs`. There is currently no migration framework — just `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`. For destructive changes you'll need to add an explicit migration step (check for column existence, `ALTER TABLE`). Keep it idempotent; `migrate()` runs on every app start.

### Regenerate the monochrome tray icon

```bash
python3 scripts/gen-tray-icon.py
```

Writes `src-tauri/icons/tray-icon.png` (44×44, grayscale+alpha). `lib.rs` bundles this file as a template image (`.icon_as_template(true)`) so macOS tints it black or white to match the menu-bar appearance. The colour `icon.png` is the *app* (Dock/Finder) icon and is configured via `tauri.conf.json` → `bundle.icon`; do not conflate the two.

## Conventions

- **Error handling.** Rust side: prefer `AppResult<T>` and the `?` operator. Surface human-readable messages via `AppError::Other(...)` only when no better variant exists.
- **Logging.** Currently there is none. If you add some, use `tracing` (already in the dep tree as a transitive) rather than `println!`, and gate verbose output behind an env var.
- **Async.** Use `tokio::sync::Mutex` for shared state, never `std::sync::Mutex` (Tauri commands are async and holding a std mutex across an `.await` is undefined behaviour).
- **Frontend.** No frameworks. Use `document.querySelector` and direct DOM manipulation. The codebase is intentionally small and dependency-light — keep it that way.
- **Comments.** Sparse. Explain *why* (constraints, gotchas, references to platform quirks), not *what*. The code is short enough that "what" is obvious.

## Things that have bitten people before

- **Stale Cargo cache after moving the repo.** If you see `failed to read plugin permissions: failed to read file '/some/old/path/...autogenerated/...'`, run `cargo clean` in `src-tauri/`. Tauri's codegen embeds absolute paths into the build cache.
- **Tray icon invisible.** Two distinct failure modes: (a) the icon image has empty alpha and is used as a template (renders nothing), (b) the tray is being hidden by code rather than just the dock. Both have happened. The fix in both cases is in `lib.rs` `setup()` and `apply_dock_mode()`.
- **Default terminal returns "Terminal" even when iTerm is default.** The user has never explicitly set a handler for the `com.apple.terminal.shell-script` UTI. Tell them to right-click any `.command` file → **Get Info** → **Open with...** → set + **Change All...**.
- **`open -b <bundle>` succeeds but nothing happens.** macOS Automation permission likely missing. System Settings → Privacy & Security → Automation → enable SSH Bookmarker → terminal of choice.

## Don't

- Don't call `ssh` directly. Use the script-based launcher.
- Don't add a network call from the backend without a very good reason. This app is local-first by design; that is a selling point.
- Don't store passwords. The schema has no field for them and the README explicitly tells users not to.
- Don't introduce a frontend framework "for ergonomics". The DOM is tiny.
- Don't break the menu-bar always-visible invariant. (See above.)
- Don't commit `dist/`, `src-tauri/target/`, or `src-tauri/gen/schemas/`.

## When you finish a change

- `cd src-tauri && cargo check` — fast feedback on Rust errors.
- `npm run tauri dev` — exercise the UI path your change touches.
- `npm run tauri build` — make sure the bundle still produces. The bundler runs `bundle_dmg.sh`, which is slow but catches a class of icon/permission issues that `cargo build` misses.

## Repo metadata

- Default branch: `main`
- Remote: `https://github.com/aletheia/ssh-bookmarker` (private)
- Bundle identifier: `com.aletheia.sshbookmarker`
