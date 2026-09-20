#!/usr/bin/env bash
# Dev runner: vite dev server (tauri devUrl) + the shell, with cleanup.
set -euo pipefail
cd "$(dirname "$0")"

pnpm --dir ui install >/dev/null

VITE_PID=""
cleanup() {
  if [ -n "$VITE_PID" ]; then
    kill "$VITE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

echo "ruoxi: starting vite dev server on :5173"
pnpm --dir ui dev &
VITE_PID=$!

for _ in $(seq 1 40); do
  if curl -s -o /dev/null http://localhost:5173/; then
    break
  fi
  sleep 0.5
done
curl -s -o /dev/null http://localhost:5173/ || {
  echo "ruoxi: vite dev server did not come up on :5173" >&2
  exit 1
}

echo "ruoxi: starting the shell (cargo run)"
cd src-tauri
cargo run
