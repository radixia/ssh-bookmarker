import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";

interface TerminalInfo {
  name: string;
  installed: boolean;
}

interface SettingsView {
  terminal: string | null;
  db_dir: string | null;
  hide_dock_icon: boolean;
  update_mode: boolean;
  effective_db_dir: string;
  default_db_dir: string;
  detected_terminal: string;
  is_macos: boolean;
  openclaw_update_cmd: string;
}

interface Bookmark {
  id?: number | null;
  name: string;
  host: string;
  user: string;
  port: number;
  auth_type: "password" | "key";
  key_path?: string | null;
  extra_args?: string | null;
  notes?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
}

const $ = <T extends HTMLElement>(sel: string) =>
  document.querySelector(sel) as T;

const listView = () => $("#list-view");
const formView = () => $("#form-view");
const listEl = () => $<HTMLDivElement>("#bookmark-list");
const emptyEl = () => $<HTMLDivElement>("#empty-state");
const form = () => $<HTMLFormElement>("#bookmark-form");
const formTitle = () => $<HTMLHeadingElement>("#form-title");
const previewEl = () => $<HTMLElement>("#cmd-preview");
const keyPathField = () => $<HTMLLabelElement>("#key-path-field");
const terminalHint = () => $<HTMLSpanElement>("#terminal-hint");

let toastTimer: number | null = null;

function toast(msg: string, isError = false) {
  const el = $<HTMLDivElement>("#toast");
  el.textContent = msg;
  el.classList.toggle("error", isError);
  el.classList.remove("hidden");
  if (toastTimer) window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => el.classList.add("hidden"), 2400);
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

function showList() {
  listView().classList.remove("hidden");
  formView().classList.add("hidden");
}

function showForm() {
  formView().classList.remove("hidden");
  listView().classList.add("hidden");
  (form().querySelector('[name="name"]') as HTMLInputElement)?.focus();
}

function readForm(): Bookmark {
  const f = form();
  const fd = new FormData(f);
  const idStr = (fd.get("id") as string) || "";
  return {
    id: idStr ? Number(idStr) : null,
    name: (fd.get("name") as string).trim(),
    host: (fd.get("host") as string).trim(),
    user: (fd.get("user") as string).trim(),
    port: Number(fd.get("port") || 22),
    auth_type: (fd.get("auth_type") as "password" | "key") || "password",
    key_path: ((fd.get("key_path") as string) || "").trim() || null,
    extra_args: ((fd.get("extra_args") as string) || "").trim() || null,
    notes: ((fd.get("notes") as string) || "").trim() || null,
  };
}

function writeForm(b: Bookmark | null) {
  const f = form();
  f.reset();
  (f.querySelector('[name="id"]') as HTMLInputElement).value = b?.id
    ? String(b.id)
    : "";
  (f.querySelector('[name="name"]') as HTMLInputElement).value = b?.name ?? "";
  (f.querySelector('[name="host"]') as HTMLInputElement).value = b?.host ?? "";
  (f.querySelector('[name="user"]') as HTMLInputElement).value = b?.user ?? "";
  (f.querySelector('[name="port"]') as HTMLInputElement).value = String(
    b?.port ?? 22
  );
  const auth = b?.auth_type ?? "password";
  (f.querySelector(`[name="auth_type"][value="${auth}"]`) as HTMLInputElement)
    .checked = true;
  (f.querySelector('[name="key_path"]') as HTMLInputElement).value =
    b?.key_path ?? "";
  (f.querySelector('[name="extra_args"]') as HTMLInputElement).value =
    b?.extra_args ?? "";
  (f.querySelector('[name="notes"]') as HTMLTextAreaElement).value =
    b?.notes ?? "";
  formTitle().textContent = b?.id ? "Edit bookmark" : "New bookmark";
  syncKeyPathVisibility();
  updatePreview();
}

function syncKeyPathVisibility() {
  const isKey =
    (form().querySelector('[name="auth_type"]:checked') as HTMLInputElement)
      ?.value === "key";
  keyPathField().style.display = isKey ? "" : "none";
}

function shellQuote(s: string): string {
  if (/^[A-Za-z0-9_\-/.:@]+$/.test(s)) return s;
  return `'${s.replace(/'/g, "'\\''")}'`;
}

function buildSshCommand(b: Bookmark): string {
  const parts: string[] = ["ssh"];
  if (b.port && b.port !== 22) {
    parts.push("-p", String(b.port));
  }
  if (b.auth_type === "key" && b.key_path && b.key_path.trim()) {
    parts.push("-i", shellQuote(b.key_path.trim()));
  }
  if (b.extra_args && b.extra_args.trim()) {
    parts.push(b.extra_args.trim());
  }
  parts.push(`${b.user}@${b.host}`);
  return parts.join(" ");
}

function updatePreview() {
  const b = readForm();
  if (!b.host || !b.user) {
    previewEl().textContent = "ssh user@host";
    return;
  }
  previewEl().textContent = buildSshCommand(b);
}

function bookmarkCard(b: Bookmark): HTMLDivElement {
  const card = document.createElement("div");
  card.className = "card";
  const portStr = b.port && b.port !== 22 ? `:${b.port}` : "";
  card.innerHTML = `
    <div class="card-head">
      <div class="card-title">
        <span class="led led-pending" data-led title="Checking…"></span>
        <h3 class="card-name">${escapeHtml(b.name)}</h3>
      </div>
      <div class="card-menu-wrap">
        <button class="card-menu-btn" type="button" data-action="menu" aria-label="More actions" aria-haspopup="menu" aria-expanded="false">
          <svg viewBox="0 0 4 16" width="4" height="16" aria-hidden="true">
            <circle cx="2" cy="2" r="1.6" />
            <circle cx="2" cy="8" r="1.6" />
            <circle cx="2" cy="14" r="1.6" />
          </svg>
        </button>
        <div class="card-menu hidden" role="menu">
          <button type="button" role="menuitem" data-action="edit">Edit</button>
          <button type="button" role="menuitem" data-action="duplicate">Duplicate</button>
          <button type="button" role="menuitem" class="danger" data-action="delete">Delete</button>
        </div>
      </div>
    </div>
    <div class="card-conn">${escapeHtml(b.user)}@${escapeHtml(b.host)}${portStr}</div>
    ${b.notes ? `<p class="card-notes">${escapeHtml(b.notes)}</p>` : ""}
    <div class="card-actions">
      <button class="btn btn-launch" data-action="launch">Launch SSH</button>
    </div>
  `;
  const menuBtn = card.querySelector<HTMLButtonElement>('[data-action="menu"]')!;
  const menu = card.querySelector<HTMLDivElement>(".card-menu")!;
  menuBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    const willOpen = menu.classList.contains("hidden");
    closeAllCardMenus();
    if (willOpen) {
      menu.classList.remove("hidden");
      menuBtn.setAttribute("aria-expanded", "true");
    }
  });
  card.querySelector('[data-action="launch"]')?.addEventListener("click", () =>
    launchBookmark(b)
  );
  card.querySelector('[data-action="edit"]')?.addEventListener("click", () => {
    closeAllCardMenus();
    writeForm(b);
    showForm();
  });
  card.querySelector('[data-action="duplicate"]')?.addEventListener("click", () => {
    closeAllCardMenus();
    void duplicateBookmark(b);
  });
  card.querySelector('[data-action="delete"]')?.addEventListener("click", () => {
    closeAllCardMenus();
    void deleteBookmark(b);
  });
  void checkCardReachable(card, b);
  return card;
}

function closeAllCardMenus() {
  document.querySelectorAll(".card-menu").forEach((m) => m.classList.add("hidden"));
  document
    .querySelectorAll<HTMLElement>(".card-menu-btn")
    .forEach((b) => b.setAttribute("aria-expanded", "false"));
}

async function duplicateBookmark(b: Bookmark) {
  const copy: Bookmark = {
    name: `${b.name} (copy)`,
    host: b.host,
    user: b.user,
    port: b.port,
    auth_type: b.auth_type,
    key_path: b.key_path ?? null,
    extra_args: b.extra_args ?? null,
    notes: b.notes ?? null,
  };
  try {
    await invoke("create_bookmark", { bookmark: copy });
    toast(`Duplicated "${b.name}"`);
    await refresh();
  } catch (e) {
    toast(`Duplicate failed: ${e}`, true);
  }
}

async function checkCardReachable(card: HTMLDivElement, b: Bookmark) {
  const led = card.querySelector<HTMLSpanElement>("[data-led]");
  if (!led) return;
  try {
    const ok: boolean = await invoke("check_reachable", {
      host: b.host,
      port: b.port,
    });
    led.classList.remove("led-pending");
    if (ok) {
      led.classList.add("led-up");
      led.title = `SSH responding on port ${b.port}`;
    } else {
      led.classList.add("led-down");
      led.title = `No SSH response on port ${b.port}`;
    }
  } catch (e) {
    led.classList.remove("led-pending");
    led.classList.add("led-down");
    led.title = `Check failed: ${e}`;
  }
}

async function refresh() {
  try {
    const items: Bookmark[] = await invoke("list_bookmarks");
    listEl().innerHTML = "";
    if (items.length === 0) {
      emptyEl().classList.remove("hidden");
    } else {
      emptyEl().classList.add("hidden");
      for (const b of items) {
        listEl().appendChild(bookmarkCard(b));
      }
    }
  } catch (e) {
    toast(`Failed to load bookmarks: ${e}`, true);
  }
}

async function launchBookmark(b: Bookmark) {
  if (!b.id) return;
  try {
    const cmd: string = await invoke("launch_bookmark", { id: b.id });
    toast(`Launching ${b.name}: ${cmd}`);
  } catch (e) {
    toast(`Launch failed: ${e}`, true);
  }
}

async function deleteBookmark(b: Bookmark) {
  if (!b.id) return;
  if (!window.confirm(`Delete "${b.name}"?`)) return;
  try {
    await invoke("delete_bookmark", { id: b.id });
    toast(`Deleted "${b.name}"`);
    await refresh();
  } catch (e) {
    toast(`Delete failed: ${e}`, true);
  }
}

async function saveBookmark(e: SubmitEvent) {
  e.preventDefault();
  const b = readForm();
  if (!b.name || !b.host || !b.user) {
    toast("Name, host, and user are required", true);
    return;
  }
  if (b.auth_type === "key" && !b.key_path) {
    toast("Key path is required for key auth", true);
    return;
  }
  try {
    if (b.id) {
      await invoke("update_bookmark", { bookmark: b });
      toast(`Updated "${b.name}"`);
    } else {
      await invoke("create_bookmark", { bookmark: b });
      toast(`Saved "${b.name}"`);
    }
    showList();
    await refresh();
  } catch (err) {
    toast(`Save failed: ${err}`, true);
  }
}

async function loadTerminalHint() {
  try {
    const view: SettingsView = await invoke("get_settings");
    const name = view.terminal && view.terminal.trim()
      ? view.terminal
      : view.detected_terminal;
    terminalHint().textContent = name ? `Opens in ${name}` : "";
  } catch {
    terminalHint().textContent = "";
  }
}

const prefsModal = () => $<HTMLDivElement>("#prefs-modal");
const prefsTerminalSelect = () => $<HTMLSelectElement>("#prefs-terminal");
const prefsTerminalHint = () => $<HTMLElement>("#prefs-terminal-hint");
const prefsDbDirInput = () => $<HTMLInputElement>("#prefs-db-dir");
const prefsDockField = () => $<HTMLLabelElement>("#prefs-dock-field");
const prefsHideDockCheckbox = () => $<HTMLInputElement>("#prefs-hide-dock");
const prefsUpdateModeCheckbox = () =>
  $<HTMLInputElement>("#prefs-update-mode");
const prefsUpdateCmd = () => $<HTMLElement>("#prefs-update-cmd");

let prefsState: SettingsView | null = null;

function showPrefs() {
  prefsModal().classList.remove("hidden");
}

function hidePrefs() {
  prefsModal().classList.add("hidden");
}

async function loadPrefsModal() {
  try {
    const [view, terminals] = await Promise.all([
      invoke<SettingsView>("get_settings"),
      invoke<TerminalInfo[]>("list_terminals"),
    ]);
    prefsState = view;

    const select = prefsTerminalSelect();
    select.innerHTML = "";
    const auto = document.createElement("option");
    auto.value = "";
    auto.textContent = `Auto-detect (system default: ${view.detected_terminal})`;
    select.appendChild(auto);
    for (const t of terminals) {
      const opt = document.createElement("option");
      opt.value = t.name;
      opt.textContent = t.installed ? t.name : `${t.name} (not installed)`;
      opt.disabled = !t.installed;
      select.appendChild(opt);
    }
    select.value = view.terminal ?? "";
    prefsTerminalHint().textContent = view.terminal
      ? ""
      : `Currently using ${view.detected_terminal}.`;

    prefsDbDirInput().value = view.effective_db_dir;
    prefsDockField().hidden = !view.is_macos;
    prefsHideDockCheckbox().checked = !!view.hide_dock_icon;
    prefsUpdateModeCheckbox().checked = !!view.update_mode;
    if (view.openclaw_update_cmd) {
      prefsUpdateCmd().textContent = view.openclaw_update_cmd;
    }
  } catch (e) {
    toast(`Could not load preferences: ${e}`, true);
  }
}

async function applyPrefs(next: {
  terminal?: string | null;
  db_dir?: string | null;
  hide_dock_icon?: boolean;
  update_mode?: boolean;
}) {
  if (!prefsState) return;
  const payload = {
    terminal:
      next.terminal !== undefined ? next.terminal : prefsState.terminal,
    db_dir: next.db_dir !== undefined ? next.db_dir : prefsState.db_dir,
    hide_dock_icon:
      next.hide_dock_icon !== undefined
        ? next.hide_dock_icon
        : prefsState.hide_dock_icon,
    update_mode:
      next.update_mode !== undefined
        ? next.update_mode
        : prefsState.update_mode,
  };
  try {
    const view: SettingsView = await invoke("set_settings", { settings: payload });
    prefsState = view;
    prefsDbDirInput().value = view.effective_db_dir;
    prefsTerminalSelect().value = view.terminal ?? "";
    prefsTerminalHint().textContent = view.terminal
      ? ""
      : `Currently using ${view.detected_terminal}.`;
    prefsHideDockCheckbox().checked = !!view.hide_dock_icon;
    prefsUpdateModeCheckbox().checked = !!view.update_mode;
    await loadTerminalHint();
    await refresh();
  } catch (e) {
    toast(`Could not save preferences: ${e}`, true);
  }
}

async function chooseDbDir() {
  try {
    const selected = await openDialog({
      multiple: false,
      directory: true,
      title: "Choose bookmarks folder",
    });
    if (typeof selected === "string" && selected) {
      await applyPrefs({ db_dir: selected });
      toast("Bookmarks folder updated");
    }
  } catch (e) {
    toast(`Could not choose folder: ${e}`, true);
  }
}

async function exportBookmarks() {
  try {
    const target = await saveDialog({
      title: "Export bookmarks",
      defaultPath: "ssh-bookmarks.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!target) return;
    const count: number = await invoke("export_bookmarks", { path: target });
    toast(`Exported ${count} bookmark${count === 1 ? "" : "s"}`);
  } catch (e) {
    toast(`Export failed: ${e}`, true);
  }
}

async function chooseKeyFile() {
  try {
    const home = "~";
    const selected = await openDialog({
      multiple: false,
      directory: false,
      title: "Choose SSH private key",
      defaultPath: home,
    });
    if (typeof selected === "string" && selected) {
      const input = form().querySelector(
        '[name="key_path"]'
      ) as HTMLInputElement;
      input.value = selected;
      updatePreview();
    }
  } catch (e) {
    toast(`Could not open file picker: ${e}`, true);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  $("#new-btn").addEventListener("click", () => {
    writeForm(null);
    showForm();
  });
  $("#cancel-btn").addEventListener("click", () => showList());
  $("#choose-key-btn").addEventListener("click", chooseKeyFile);

  $("#prefs-btn").addEventListener("click", async () => {
    await loadPrefsModal();
    showPrefs();
  });
  prefsModal().querySelectorAll("[data-close]").forEach((el) =>
    el.addEventListener("click", hidePrefs)
  );
  prefsTerminalSelect().addEventListener("change", () => {
    const value = prefsTerminalSelect().value;
    applyPrefs({ terminal: value === "" ? null : value });
  });
  $("#prefs-choose-folder").addEventListener("click", chooseDbDir);
  $("#prefs-reset-folder").addEventListener("click", async () => {
    await applyPrefs({ db_dir: null });
    toast("Bookmarks folder reset to default");
  });
  $("#prefs-export").addEventListener("click", exportBookmarks);
  prefsHideDockCheckbox().addEventListener("change", () => {
    void applyPrefs({ hide_dock_icon: prefsHideDockCheckbox().checked });
  });
  prefsUpdateModeCheckbox().addEventListener("change", () => {
    void applyPrefs({ update_mode: prefsUpdateModeCheckbox().checked });
  });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      if (!prefsModal().classList.contains("hidden")) hidePrefs();
      closeAllCardMenus();
    }
  });
  document.addEventListener("click", (e) => {
    const target = e.target as HTMLElement | null;
    if (!target?.closest(".card-menu-wrap")) closeAllCardMenus();
  });
  form().addEventListener("submit", saveBookmark);
  form().addEventListener("input", () => {
    syncKeyPathVisibility();
    updatePreview();
  });
  form().addEventListener("change", () => {
    syncKeyPathVisibility();
    updatePreview();
  });

  refresh();
  loadTerminalHint();
});
