APP_NAME := audiosub
DOCKER_IMAGE := $(APP_NAME)
DOCKER_TAG := latest
MODEL_URL := https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip
MODEL_DIR := models
MODEL_NAME := vosk-model-small-ru-0.22
MODEL_PATH := $(MODEL_DIR)/$(MODEL_NAME)

GREEN  := \033[0;32m
CYAN   := \033[0;36m
YELLOW := \033[1;33m
RED    := \033[0;31m
BOLD   := \033[1m
NC     := \033[0m

# Resolve model path to absolute — Vosk needs absolute path
AUDIOSUB_MODEL := $(CURDIR)/$(MODEL_DIR)/$(MODEL_NAME)

.PHONY: all build test run lint clean docker docker-build docker-run docker-build fmt check verify ci-check report help model-download

all: help

help:
	@echo "$(BOLD)$(APP_NAME)$(NC) - real-time automatic subtitles"
	@echo ""
	@echo "$(CYAN)Usage:$(NC)"
	@echo "  make $(GREEN)<target>$(NC)"
	@echo ""
	@echo "$(YELLOW)Note:$(NC) Requires libvosk.so in /home/redalexdad/.local/lib/"
	@echo "  Download from https://github.com/alphacep/vosk-api/releases"
	@echo ""
	@echo "$(CYAN)Targets:$(NC)"
	@echo "  $(GREEN)all$(NC)             Show this help"
	@echo "  $(GREEN)build$(NC)           Compile the project"
	@echo "  $(GREEN)release$(NC)         Compile in release mode"
	@echo "  $(GREEN)run [ARGS]$(NC)      Run with TUI (default, press q/Esc to quit)"
	@echo "  $(GREEN)cli [ARGS]$(NC)      Run without TUI (plain CLI mode)"
	@echo "  $(GREEN)model-download$(NC)  Download Vosk model (Russian)"
	@echo "  $(GREEN)test$(NC)            Run tests"
	@echo "  $(GREEN)check$(NC)           cargo check (fast)"
	@echo "  $(GREEN)lint$(NC)            cargo clippy"
	@echo "  $(GREEN)fmt$(NC)             Check formatting"
	@echo "  $(GREEN)verify$(NC)          Run test + check + lint + fmt (CI pipeline)"
	@echo "  $(GREEN)ci-check$(NC)       Run full CI simulation (scripts/ci-check.sh)"
	@echo "  $(GREEN)clean$(NC)           Remove build artifacts"
	@echo "  $(GREEN)docker$(NC)          Build + run via compose"
	@echo "  $(GREEN)docker-build$(NC)    Build Docker image"
	@echo "  $(GREEN)docker-run$(NC)      Run via docker compose"
	@echo "  $(GREEN)report$(NC)          Create report template"
	@echo "  $(GREEN)help$(NC)            Show this message"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make model-download"
	@echo "  make run"
	@echo "  make run ARGS='-- --list-devices'"
	@echo "  make cli"
	@echo "  make docker"

build:
	@echo "$(CYAN)→ Building $(APP_NAME)...$(NC)"
	cargo build
	@echo "$(GREEN)✓ Build complete$(NC)"

release:
	@echo "$(CYAN)→ Building $(APP_NAME) (release)...$(NC)"
	cargo build --release
	mkdir -p release
	cp target/release/audiosub release/audiosub
	@echo "$(GREEN)✓ Release build complete — release/audiosub$(NC)"

model-download:
	@echo "$(CYAN)→ Downloading $(MODEL_NAME)...$(NC)"
	mkdir -p $(MODEL_DIR)
	curl -L -o /tmp/$(MODEL_NAME).zip "$(MODEL_URL)" && \
	unzip -qo /tmp/$(MODEL_NAME).zip -d $(MODEL_DIR) && \
	rm /tmp/$(MODEL_NAME).zip && \
	echo "$(GREEN)✓ Model downloaded to $(MODEL_PATH)$(NC)" && \
	echo "$(YELLOW)  Edit .env to change AUDIOSUB_MODEL path$(NC)"

test:
	@echo "$(CYAN)→ Running tests...$(NC)"
	cargo test
	@echo "$(GREEN)✓ Tests passed$(NC)"

run:
	@echo "$(CYAN)→ Starting $(APP_NAME) (TUI mode)...$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib cargo run -- $(ARGS)

cli:
	@echo "$(CYAN)→ Starting $(APP_NAME) (CLI mode)...$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib cargo run -- --no-tui $(ARGS)

check:
	@echo "$(CYAN)→ Checking...$(NC)"
	cargo check

lint:
	@echo "$(CYAN)→ Linting...$(NC)"
	cargo clippy -- -D warnings
	@echo "$(GREEN)✓ Lint passed$(NC)"

fmt:
	@echo "$(CYAN)→ Checking formatting...$(NC)"
	cargo fmt --check
	@echo "$(GREEN)✓ Formatting OK$(NC)"

verify:
	@echo "$(CYAN)→ Verifying (test + check + lint + fmt)...$(NC)"
	$(MAKE) test && \
	$(MAKE) check && \
	$(MAKE) lint && \
	$(MAKE) fmt
	@echo "$(GREEN)✓ All checks passed$(NC)"

ci-check:
	@echo "$(CYAN)→ Running CI simulation...$(NC)"
	@scripts/ci-check.sh

clean:
	@echo "$(YELLOW)← Cleaning...$(NC)"
	cargo clean
	@echo "$(GREEN)✓ Cleaned$(NC)"

docker-build:
	@echo "$(CYAN)→ Building Docker image...$(NC)"
	DOCKER_BUILDKIT=0 docker build --network host -t $(DOCKER_IMAGE):$(DOCKER_TAG) .
	@echo "$(GREEN)✓ Docker image built$(NC)"

docker-run:
	@echo "$(CYAN)→ Starting Docker Compose...$(NC)"
	USER_ID=$(shell id -u) GROUP_ID=$(shell id -g) docker compose up

docker: docker-run

report:
	@echo "$(CYAN)→ Creating report...$(NC)"
	@mkdir -p reports
	@echo "# Report $(shell date -u '+%Y-%m-%d %H:%M:%S UTC')" > reports/latest.md
	@echo "" >> reports/latest.md
	@echo "## Changes" >> reports/latest.md
	@echo "- " >> reports/latest.md
	@echo "" >> reports/latest.md
	@echo "## Problems" >> reports/latest.md
	@echo "- " >> reports/latest.md
	@echo "" >> reports/latest.md
	@echo "## Solutions" >> reports/latest.md
	@echo "- " >> reports/latest.md
	@echo "" >> reports/latest.md
	@echo "---" >> reports/latest.md
	@echo "Report generated automatically." >> reports/latest.md
	@echo "$(GREEN)✓ Created reports/latest.md$(NC) — edit it before committing."
