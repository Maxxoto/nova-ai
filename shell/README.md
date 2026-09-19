# Ruoxi Shell (Tauri 2) — M0 scaffold

Tray-resident desktop shell per [RFC-0002](../../docs/rfc/RFC-0002-platform-shell.md).
Build plan workstreams W1 + W5 (see [plans/m0-build-plan.md](../../plans/m0-build-plan.md)).

## Run (dev)

```bash
cd shell/src-tauri
cargo run
```

The shell expects the Python brain sidecar to be launchable as
`python3 -m app.interfaces.sidecar` (override via `settings.json`:
`sidecar_command` / `sidecar_args`). From the repo root, that means `src/`
must be on `PYTHONPATH`, e.g.:

```bash
PYTHONPATH=src cargo run   # from shell/src-tauri
```

## Linux system prerequisites (build-time)

Tauri 2 needs WebKit/GTK dev packages to compile:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## Status (M0)

- W1: tray + menu (Show result panel / quit / pause captures / about),
  single-instance lock, accessory activation policy, JSON settings persistence.
- W4 (UI): result panel window + components per `docs/DESIGN.md` —
  frameless, transparent, always-on-top, 424x480, hidden at launch; the
  `panel` webview loads `../ui` at `?view=panel` (panel only, transparent
  page, OS light/dark). Esc hides it (`hide_panel` command; AC-06). Capture
  and IPC wiring still pending.
- W5 (Rust side): sidecar supervisor — spawn, periodic JSON-RPC `ping`
  health-check, exponential backoff restarts, degraded tooltip after
  repeated failures. Python counterpart: `src/app/interfaces/sidecar/`.
- Icons: tray icon is generated at runtime; bundle icons arrive with W6
  packaging (`bundle.active = false` for now).
