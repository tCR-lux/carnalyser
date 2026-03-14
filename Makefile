# =============================================================================
# ECU Analyzer — Makefile
# Copyright (c) 2026 Cédric Renzi — GPL-3.0-only / Commercial License
# =============================================================================

BINARY    := ecu-analyzer
CARGO     := cargo
TARGET    := target/release/$(BINARY)
TARGET_DBG:= target/debug/$(BINARY)

# ── Default ──────────────────────────────────────────────────────────────────
.DEFAULT_GOAL := help

.PHONY: help build build-debug run run-debug test lint fmt fmt-check clean         check audit doc release install

# ── Help ─────────────────────────────────────────────────────────────────────
help:
	@echo ""
	@echo "  ECU Analyzer — build helper"
	@echo "  Copyright (c) 2026 Cédric Renzi"
	@echo ""
	@echo "  Usage: make <target>"
	@echo ""
	@echo "  Build"
	@echo "    build          Optimised release build"
	@echo "    build-debug    Debug build (faster compile, slower binary)"
	@echo "    check          Type-check without producing a binary"
	@echo ""
	@echo "  Run"
	@echo "    run            Build (release) + launch the GUI"
	@echo "    run-debug      Build (debug)   + launch the GUI"
	@echo ""
	@echo "  Test & Quality"
	@echo "    test           Run all unit tests"
	@echo "    lint           Run clippy (warnings as errors)"
	@echo "    fmt            Auto-format all source files"
	@echo "    fmt-check      Check formatting without modifying files"
	@echo "    audit          Check dependencies for known CVEs"
	@echo ""
	@echo "  Docs & Install"
	@echo "    doc            Build + open rustdoc in browser"
	@echo "    install        Install binary to ~/.cargo/bin/"
	@echo "    release        Full pipeline: fmt-check lint test build"
	@echo ""
	@echo "    clean          Remove all build artifacts"
	@echo ""

# ── Build ─────────────────────────────────────────────────────────────────────
build:
	$(CARGO) build --release

build-debug:
	$(CARGO) build

check:
	$(CARGO) check

# ── Run ──────────────────────────────────────────────────────────────────────
run: build
	$(TARGET)

run-debug: build-debug
	$(TARGET_DBG)

# ── Test ─────────────────────────────────────────────────────────────────────
test:
	$(CARGO) test -- --nocapture

# ── Quality ──────────────────────────────────────────────────────────────────
lint:
	$(CARGO) clippy -- -D warnings

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt -- --check

audit:
	@command -v cargo-audit >/dev/null 2>&1 || 		{ echo "Installing cargo-audit..."; $(CARGO) install cargo-audit; }
	$(CARGO) audit

# ── Docs ─────────────────────────────────────────────────────────────────────
doc:
	$(CARGO) doc --open --no-deps

# ── Install ──────────────────────────────────────────────────────────────────
install:
	$(CARGO) install --path .

# ── Full release pipeline ────────────────────────────────────────────────────
release: fmt-check lint test build
	@echo ""
	@echo "  ✅  Release build ready: $(TARGET)"
	@echo ""

# ── Clean ────────────────────────────────────────────────────────────────────
clean:
	$(CARGO) clean
	@echo "  🧹  Build artifacts removed."
