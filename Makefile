SHELL := /bin/sh
.DEFAULT_GOAL := build

CLIENT_DIR := client
GO_SERVER_DIR := server/go_server
RUST_SERVER_DIR := server/rust_server

CARGO ?= cargo
GO ?= go

RED := \033[31m
CYAN := \033[36m
RESET := \033[0m

CLIENT_ARGS ?=
GO_SERVER_ARGS ?=
RUST_SERVER_ARGS ?=

RUN_DIR ?= /tmp/the_answer_protocol-$(shell id -u)
GO_SERVER_PID_FILE := $(RUN_DIR)/go-server.pid
RUST_SERVER_PID_FILE := $(RUN_DIR)/rust-server.pid
GO_SERVER_LOG := $(RUN_DIR)/go-server.log
RUST_SERVER_LOG := $(RUN_DIR)/rust-server.log

HELPERS = \
	error_log() { printf '$(RED)error: %s$(RESET)\n' "$$*" >&2; }; \
	info_log() { printf '$(CYAN)info: %s$(RESET)\n' "$$*"; }; \
	ensure_cargo() { command -v "$(CARGO)" >/dev/null 2>&1 || { error_log "cargo is required"; return 1; }; }; \
	ensure_go() { command -v "$(GO)" >/dev/null 2>&1 || { error_log "go is required"; return 1; }; }; \
	ensure_clang() { test -x /usr/bin/clang || { error_log "/usr/bin/clang is required by the sandbox"; return 1; }; }; \
	ensure_bwrap() { test -x /usr/bin/bwrap || { error_log "/usr/bin/bwrap is required by the sandbox"; return 1; }; }; \
	ensure_clippy() { command -v cargo-clippy >/dev/null 2>&1 || { error_log "cargo-clippy is required"; return 1; }; }; \
	ensure_rustfmt() { command -v rustfmt >/dev/null 2>&1 || { error_log "rustfmt is required"; return 1; }; }; \
	ensure_gui() { test -f "$(CLIENT_DIR)/gui/Cargo.toml" || { error_log "the gui client is not present on this branch"; return 1; }; }; \
	ensure_stopped() { pid_file="$$1"; name="$$2"; test -f "$$pid_file" || return 0; pid=$$(cat "$$pid_file"); case "$$pid" in ''|*[!0-9]*) rm -f "$$pid_file"; return 0;; esac; kill -0 "$$pid" 2>/dev/null || { rm -f "$$pid_file"; return 0; }; error_log "$$name is already running (PID $$pid)"; return 1; }; \
	stop_process() { pid_file="$$1"; name="$$2"; expected_exe="$$3"; test -f "$$pid_file" || { info_log "$$name is not running"; return 0; }; pid=$$(cat "$$pid_file"); case "$$pid" in ''|*[!0-9]*) error_log "invalid PID for $$name"; return 1;; esac; kill -0 "$$pid" 2>/dev/null || { rm -f "$$pid_file"; info_log "$$name is not running"; return 0; }; actual_exe=$$(readlink "/proc/$$pid/exe" 2>/dev/null); test "$$actual_exe" = "$$expected_exe" || { error_log "refusing to stop $$name: PID $$pid belongs to another process"; return 1; }; kill "$$pid" || { error_log "could not stop $$name (PID $$pid)"; return 1; }; rm -f "$$pid_file"; info_log "$$name stopped"; };

# GLOBAL

.PHONY: install build \
	build-go-server build-rust-server build-client-tui build-client-gui \
	run-go-server run-rust-server run-client-tui run-client-gui \
	lint-client lint-go-server lint-rust-server \
	run stop lint clean

install:
	@$(HELPERS) \
		info_log "checking required tools" && \
		ensure_cargo && \
		ensure_go && \
		ensure_clang && \
		ensure_bwrap && \
		ensure_clippy && \
		ensure_rustfmt
	@$(HELPERS) info_log "fetching client dependencies"
	@cd $(CLIENT_DIR) && $(CARGO) fetch --locked
	@$(HELPERS) info_log "fetching Rust server dependencies"
	@cd $(RUST_SERVER_DIR) && $(CARGO) fetch --locked
	@$(HELPERS) info_log "fetching Go server dependencies"
	@cd $(GO_SERVER_DIR) && $(GO) mod download

build: build-go-server build-rust-server build-client-tui build-client-gui
	@$(HELPERS) info_log "all components built"

run:
	@mkdir -p "$(RUN_DIR)"
	@$(HELPERS) \
		ensure_stopped "$(GO_SERVER_PID_FILE)" "go server" && \
		ensure_stopped "$(RUST_SERVER_PID_FILE)" "rust server"
	@$(MAKE) build-go-server build-rust-server build-client-tui
	@$(HELPERS) info_log "starting Rust server in background"
	@(cd $(RUST_SERVER_DIR) && exec ./target/debug/rust_server) > "$(RUST_SERVER_LOG)" 2>&1 & \
		echo $$! > "$(RUST_SERVER_PID_FILE)"
	@$(HELPERS) info_log "starting Go server in background"
	@(cd $(GO_SERVER_DIR) && exec ./go_server $(GO_SERVER_ARGS)) > "$(GO_SERVER_LOG)" 2>&1 & \
		echo $$! > "$(GO_SERVER_PID_FILE)"
	@$(HELPERS) info_log "starting TUI client"
	@cd $(CLIENT_DIR) && exec ./target/debug/tui $(CLIENT_ARGS)

stop:
	@$(HELPERS) \
		stop_process "$(GO_SERVER_PID_FILE)" "go server" "$(abspath $(GO_SERVER_DIR)/go_server)"; \
		go_status=$$?; \
		stop_process "$(RUST_SERVER_PID_FILE)" "rust server" "$(abspath $(RUST_SERVER_DIR)/target/debug/rust_server)"; \
		rust_status=$$?; \
		test $$go_status -eq 0 && test $$rust_status -eq 0

lint: lint-client lint-go-server lint-rust-server
	@$(HELPERS) info_log "all components linted"

clean:
	@$(HELPERS) info_log "cleaning build artifacts"
	@cd $(GO_SERVER_DIR) && $(GO) clean
	@cd $(CLIENT_DIR) && $(CARGO) clean
	@cd $(RUST_SERVER_DIR) && $(CARGO) clean

# CLIENT

build-client-tui:
	@$(HELPERS) ensure_cargo && info_log "building TUI client"
	@cd $(CLIENT_DIR) && $(CARGO) build --package tui

build-client-gui:
	@$(HELPERS) ensure_cargo && ensure_gui && info_log "building GUI client"
	@cd $(CLIENT_DIR) && $(CARGO) build --package gui

run-client-tui: build-client-tui
	@$(HELPERS) info_log "running TUI client"
	@cd $(CLIENT_DIR) && ./target/debug/tui $(CLIENT_ARGS)

run-client-gui: build-client-gui
	@$(HELPERS) info_log "running GUI client"
	@cd $(CLIENT_DIR) && ./target/debug/gui $(CLIENT_ARGS)

lint-client:
	@$(HELPERS) ensure_cargo && ensure_clippy && ensure_rustfmt && info_log "linting clients"
	@cd $(CLIENT_DIR) && $(CARGO) clippy --all-targets --workspace
	@cd $(CLIENT_DIR) && $(CARGO) fmt --all --check

# GO SERVER

build-go-server:
	@$(HELPERS) ensure_go && info_log "building Go server"
	@cd $(GO_SERVER_DIR) && $(GO) build

run-go-server: build-go-server
	@$(HELPERS) info_log "running Go server"
	@cd $(GO_SERVER_DIR) && ./go_server $(GO_SERVER_ARGS)

lint-go-server:
	@$(HELPERS) ensure_go && info_log "linting Go server"
	@$(HELPERS) files=$$(cd $(GO_SERVER_DIR) && gofmt -l .); test -z "$$files" || { error_log "Go files are not formatted: $$files"; exit 1; }
	@cd $(GO_SERVER_DIR) && $(GO) vet ./...

# RUST SERVER

build-rust-server:
	@$(HELPERS) ensure_cargo && info_log "building Rust server"
	@cd $(RUST_SERVER_DIR) && $(CARGO) build

run-rust-server: build-rust-server
	@$(HELPERS) info_log "running Rust server"
	@cd $(RUST_SERVER_DIR) && ./target/debug/rust_server $(RUST_SERVER_ARGS)

lint-rust-server:
	@$(HELPERS) ensure_cargo && ensure_clippy && ensure_rustfmt && info_log "linting Rust server"
	@cd $(RUST_SERVER_DIR) && $(CARGO) clippy --all-targets
	@cd $(RUST_SERVER_DIR) && $(CARGO) fmt --all --check
