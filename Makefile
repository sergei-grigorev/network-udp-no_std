# Makefile for Rust project

.PHONY: help format lint

help: ## Show all available commands
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## ' Makefile | awk -F ':|##' '{printf "  make %-10s %s\n", $$1, $$3}'

format: ## Format the code using rustfmt
	cargo fmt --all

lint: ## Run linter (clippy) to check code
	cargo clippy --all-targets --all-features -- -D warnings

run-server: ## Run the server
	cargo run --bin server

run-client: ## Run the client
	cargo run --bin client