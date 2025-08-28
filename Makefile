SHELL := /usr/bin/bash

.PHONY: all rust-setup rust-check server-run ui-install ui-dev tune-setup tune-run

all: rust-check

rust-setup:
	curl -sSf https://sh.rustup.rs | sh -s -- -y

rust-check:
	cd rust && cargo check

server-run:
	cd rust && cargo run -p agent-server

ui-install:
	cd ui && npm install --no-audit --no-fund

ui-dev:
	cd ui && npm run dev

tune-setup:
	cd tuning && python3 -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt

tune-run:
	cd tuning && source .venv/bin/activate && python train_lora.py --help

