# Contributing to SSH Bookmarker

Thanks for thinking about contributing! This is a small, focused app — keeping it that way is a feature, so please read this guide before sinking time into a large change.

## TL;DR

- File an issue first for anything bigger than a one-line fix.
- Read [CLAUDE.md](CLAUDE.md) before editing the Rust side — it documents the non-obvious invariants that have bitten people.
- `cd src-tauri && cargo check` and `npm run tauri dev` must both succeed locally.
- Open a PR against `main` with a clear description and a test plan.

## Ground rules

SSH Bookmarker is intentionally minimal:

- **No frameworks on the frontend.** The DOM is small; please don't add React, Vue, Svelte, jQuery, etc.
- **Local-first.** No network calls from the backend (the reachability TCP probe is the only exception, and it talks to the user's own hosts).
- **No password storage.** The schema has no field for credentials and we won't add one. Use key-based auth.
- **macOS-only for now.** Cross-platform work is welcome, but please discuss the approach in an issue first — the launcher abstraction will need rethinking.

If you're proposing a feature that conflicts with any of the above, open an issue so we can talk before you write code.

## Getting set up

Prerequisites:

- macOS 11+ (Apple silicon or Intel)
- Node.js 20+
- Rust toolchain (`rustup`, stable)
- Xcode Command Line Tools

```bash
git clone https://github.com/aletheia/ssh-bookmarker.git
cd ssh-bookmarker
npm install
npm run tauri dev
```

The dev server runs at `http://localhost:1420`; the Rust side recompiles on save.

## Project layout

The full map lives in [CLAUDE.md](CLAUDE.md). Quick summary:

- `src/` — TypeScript frontend (Vite, no framework).
- `src-tauri/src/` — Rust backend (Tauri commands, libSQL access, launcher, settings).
- `src-tauri/icons/` — bundled icons. Note that `icon.png` is the *app* icon (Dock/Finder) and `tray-icon.png` is the monochrome template for the menu bar — don't conflate them.
- `scripts/gen-tray-icon.py` — regenerates the monochrome tray icon.

## Workflow

1. **Open an issue** describing the bug or feature. For tiny fixes (typos, obvious bugs), a PR alone is fine.
2. **Fork and branch** from `main`. Use a short kebab-case branch name: `fix/tray-visibility`, `feat/wsl-launcher`.
3. **Make focused commits.** Prefer several small commits over one giant one. Commit messages should explain *why*, not just *what*.
4. **Run the checks** listed below before opening a PR.
5. **Open a PR.** Reference the issue (`Fixes #123`), summarise the change, and include a short test plan describing what you actually exercised.

## Pre-flight checks

Before you push:

```bash
# Rust side compiles cleanly
cd src-tauri && cargo check

# Full build (catches icon/permissions issues that cargo check misses)
npm run tauri build

# Exercise your change end-to-end in dev mode
npm run tauri dev
```

If you touched the launcher or terminal-detection code, please verify the actual launch path in at least two terminal apps (Terminal + iTerm is a good combination).

## Coding conventions

### Rust

- Use `AppResult<T>` and the `?` operator. Don't return `Result<T, String>` from Tauri commands — `AppError` already serialises cleanly to the frontend.
- Async state goes in `tokio::sync::Mutex`, never `std::sync::Mutex`. Holding a std mutex across an `.await` is undefined behaviour.
- New Tauri commands must be added to the `tauri::generate_handler!(...)` list at the bottom of `run()`. If you forget, the command silently 404s.
- Any command that mutates bookmarks must call `rebuild_tray_menu(&app).await` before returning, so the menu stays in sync.
- Don't shell out to `ssh` directly. The launcher writes a self-deleting `.command` script and hands it to `open -b <bundle-id>` — that path keeps macOS sandboxing and Automation permissions clean.

### TypeScript

- Plain DOM, no framework. `document.querySelector` is fine.
- All IPC goes through `invoke<ReturnType>("snake_case_name", { ... })`.
- Keep the surface small: shared helpers live at the top of `src/main.ts`. There's no module split because there isn't enough code to justify one.

### Commits

- One logical change per commit.
- Subject line ≤ 72 chars, imperative mood ("Add reachability probe", not "Added reachability probe").
- Body explains the *why* and references issues with `Fixes #N` / `Refs #N`.

## Things to read before touching certain areas

| If you're changing...           | Read first                                            |
|---------------------------------|-------------------------------------------------------|
| The tray menu or dock policy    | [CLAUDE.md → Key invariants](CLAUDE.md)              |
| The DB schema                   | [CLAUDE.md → Change the DB schema](CLAUDE.md)        |
| Terminal launching              | `src-tauri/src/launcher.rs` + macOS sandbox docs     |
| Settings persistence            | `src-tauri/src/settings.rs`                          |
| Adding a new Tauri command      | [CLAUDE.md → Add a new Tauri command](CLAUDE.md)     |

## Releasing (maintainers)

1. Bump `version` in both [`package.json`](package.json) and [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json). They must match.
2. Update any user-visible changes in the README if relevant.
3. `npm run tauri build` and verify both bundles (`.app` and `.dmg`) open cleanly.
4. Tag: `git tag v<version> && git push --tags`.
5. Create a GitHub release; attach the DMG.

## Code of conduct

Be kind, be specific, assume good intent. We don't have a formal CoC yet; if behaviour becomes an issue we'll adopt the [Contributor Covenant](https://www.contributor-covenant.org/).

## Reporting security issues

Please **do not** open a public issue for security problems. Email the maintainer directly (see commit author lines for the address) with a clear reproduction.

## License

By contributing, you agree that your contributions will be licensed under the same [MIT License](LICENSE) that covers the project.
