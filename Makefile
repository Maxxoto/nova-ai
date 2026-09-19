# Makefile for the Ruòxī / nova-ai monorepo

.PHONY: help install test lint format clean shell shell-dev shell-ui shell-test run-cli

help:
	@echo "Available commands:"
	@echo "  install    - Install Python deps (uv sync) + UI deps (pnpm install)"
	@echo "  test       - Run Python test suite"
	@echo "  shell      - ONE COMMAND: install+build UI, then run the tray shell (static, no hot reload)"
	@echo "  shell-dev  - Run tray shell in dev mode (Vite hot-reload + tauri dev)"
	@echo "  shell-ui   - Typecheck + build the panel UI only"
	@echo "  shell-test - Run the Rust shell test suite"
	@echo "  run-cli    - Run the Python brain CLI chat"
	@echo "  lint       - Run linting"
	@echo "  format     - Format code"
	@echo "  clean      - Clean build artifacts"

install:
	uv sync
	pnpm --dir shell/ui install

test:
	uv run pytest

shell:
	pnpm --dir shell/ui install
	pnpm --dir shell/ui build
	cd shell/src-tauri && cargo run

shell-dev:
	pnpm --dir shell/ui install
	cd shell/src-tauri && npx --yes @tauri-apps/cli@2 dev

shell-ui:
	pnpm --dir shell/ui build

shell-test:
	cd shell/src-tauri && cargo test

run-cli:
	uv run nova

lint:
	uv run flake8 src/ tests/

format:
	uv run black src/ tests/

clean:
	rm -rf build/ dist/ *.egg-info
	find . -type f -name "*.pyc" -delete
	find . -type d -name "__pycache__" -delete
