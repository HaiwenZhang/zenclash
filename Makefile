.PHONY: help build test check fmt clippy clean install-deps run

help:
	@echo "ZenClash Makefile Commands:"
	@echo "  make build         - Build the project in release mode"
	@echo "  make build-dev    - Build the project in debug mode"
	@echo "  make test         - Run all tests"
	@echo "  make check        - Run cargo check"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make clippy       - Run clippy lints"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make install-deps - Install system dependencies"
	@echo "  make run          - Run the application"

build:
	cargo build --release

build-dev:
	cargo build

test:
	cargo test

test-release:
	cargo test --release

check:
	cargo check

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

clippy:
	cargo clippy -- -D warnings

clippy-fix:
	cargo clippy --fix --allow-dirty

clean:
	cargo clean

install-deps:
	./scripts/install-deps.sh

run:
	cargo run -p zenclash-cli

all: check fmt clippy test

.DEFAULT_GOAL := help
