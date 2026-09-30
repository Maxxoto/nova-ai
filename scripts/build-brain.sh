#!/usr/bin/env bash
# Freezes the Python brain (stdio JSON-RPC sidecar) into a self-contained
# onedir binary the DMG ships as an app resource — no Python needed on the
# target Mac. PyInstaller runs through `uv run --with` so the project venv
# stays untouched while import tracing still sees the repo's dependencies.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="shell/src-tauri/resources/brain"
rm -rf "$OUT"

uv run --with pyinstaller pyinstaller \
  --noconfirm --clean \
  --name brain \
  --onedir \
  --paths src \
  --collect-all litellm \
  --exclude-module logfire \
  --collect-all tiktoken \
  --collect-all tiktoken_ext \
  src/app/interfaces/sidecar/__main__.py \
  --distpath shell/src-tauri/resources

du -sh "$OUT"
"$OUT/brain/brain" --version >/dev/null 2>&1 || true
echo "brain frozen at $OUT/brain"
