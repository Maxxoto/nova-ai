# Ruoxi Shell (Tauri 2)

Tray-resident desktop shell per [RFC-0002](../../docs/rfc/RFC-0002-platform-shell.md).
Build plan workstreams W1–W6 (see [plans/m0-build-plan.md](../../plans/m0-build-plan.md));
the design contract lives in [docs/DESIGN.md](../../docs/DESIGN.md).

## Run (dev)

One command from the repo root installs deps, builds the UI, and runs the tray shell (static build, no hot reload):

```bash
make shell
```

For dev mode, `make shell-dev` starts the Vite dev server on
http://localhost:5173 (HMR) alongside `tauri dev`: UI edits hot-reload, Rust
edits rebuild and relaunch the app.

```bash
make shell-dev
```

The shell spawns the Python brain sidecar (`python3 -m app.interfaces.sidecar`
by default; override `sidecar_command` / `sidecar_args` in the app's
`settings.json`). The module must be importable — point `sidecar_command` at the
repo venv, or export `PYTHONPATH`:

```bash
PYTHONPATH=src cargo run   # from shell/src-tauri
```

The dev/build hooks in `tauri.conf.json` use an explicit cwd, so `tauri dev` /
`tauri build` work from any directory.

First run: grant **Screen Recording** (without it macOS hands the app a wallpaper-only
image — no other apps' windows) and **Accessibility** (hold-to-talk). The setup ritual
asks with a why-line first; `Settings → Permissions` opens the matching System Settings
panes.

## Package (unsigned .dmg)

```bash
cd shell/src-tauri && npx --yes @tauri-apps/cli@2 build
# → target/release/bundle/dmg/Ruoxi_0.1.0_aarch64.dmg
```

## Linux system prerequisites (build-time)

Tauri 2 needs WebKit/GTK dev packages to compile:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## Status (M0)

- **W1 — tray + lifecycle:** menu (Show result panel · Pause captures · Offline
  mode · Settings… · Open timeline · About · Quit), single-instance lock,
  accessory activation policy, JSON settings (`offline` defaults on;
  `read_aloud` / `answer_length` / `theme` / `default_scope` / `ptt_hotkey`
  alongside the sidecar config).
- **W2 — hotkeys + push-to-talk:** capture hotkeys `Alt+Shift+R/W/F`; PTT `F8`
  by default, rebindable in Settings → Voice & answers (validated against
  capture-hotkey conflicts; applies on the next app start).
- **W3 — capture + overlay:** fullscreen (display under cursor), window
  (frontmost non-self), region via the in-app overlay (dim, sharp selection,
  handles, live dimension chip; the overlay hides before pixels are grabbed).
  Capture store v0 (date-sharded files + SQLite index, sha256 dedupe). Tray
  mirrors paused/offline states; macOS ships the template icon variants.
- **W4 — UI surfaces** (`shell/ui`, `index.html?view=…`): `panel` (live
  capture → thinking → streaming → complete/error; Esc aborts/stops the
  read-aloud, ⌥⇧D hides, AC-06),
  `onboarding` (five-step why-line ritual + guided first capture, AC-12),
  `settings` (offline banner + seven sections incl. permissions and re-run
  ritual), `timeline` (real store: stats, day groups, thumbnails,
  typed-confirm delete), `overlay`. Theme is centralised (`dawn`/`night`/`system`).
- **W5 — brain sidecar:** supervisor spawn + JSON-RPC `ping` with exponential
  backoff; `session.ask` / `session.abort` stream to the panel through
  `panel:*` events. Python counterpart: `src/app/interfaces/sidecar/`.
- **W6 — packaging:** unsigned `.dmg` (see above); icon set generated from
  `assets/Ruoxi Circle.png` (upscaled — swap in a ≥1024² source before
  release). Open gap: the sidecar expects a Python with the repo deps (e.g.
  `.venv/bin/python` via `sidecar_command`); bundling a runtime is not done yet.
- **Design check:** `python3 scripts/verify_design_md.py` re-validates tokens
  and contrast (currently `ALL CHECKS PASSED`).
