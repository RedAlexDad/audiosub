APP_NAME := audiosub
DOCKER_IMAGE := $(APP_NAME)
DOCKER_TAG := latest

# --- Vosk ---
VOSK_MODEL_URL := https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip
VOSK_MODEL_NAME := vosk-model-small-ru-0.22

# --- Whisper ---
WHISPER_MODEL_URL := https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
WHISPER_MODEL_NAME := ggml-base.bin

MODEL_DIR := models
MODEL_PATH := $(MODEL_DIR)/$(VOSK_MODEL_NAME)

GREEN  := \033[0;32m
CYAN   := \033[0;36m
YELLOW := \033[1;33m
RED    := \033[0;31m
BOLD   := \033[1m
NC     := \033[0m

# Resolve model path to absolute — Vosk needs absolute path
AUDIOSUB_MODEL := $(CURDIR)/$(MODEL_DIR)/$(MODEL_NAME)

.PHONY: all build build-whisper build-both release release-whisper release-both release-linux release-win release-mac model-download model-whisper test run run-whisper run-both cli cli-whisper check lint fmt verify ci-check clean docker docker-build docker-run report help

all: help

help:
	@echo "$(BOLD)$(APP_NAME)$(NC) - real-time automatic subtitles"
	@echo ""
	@echo "$(CYAN)Использование:$(NC)"
	@echo "  make $(GREEN)<цель>$(NC)"
	@echo ""
	@echo "$(YELLOW)Примечания:$(NC)"
	@echo "  Vosk:   Требуется libvosk.so в /home/redalexdad/.local/lib/"
	@echo "          Скачать: https://github.com/alphacep/vosk-api/releases"
	@echo "  Whisper: Нужен WHISPER_DONT_GENERATE_BINDINGS=1, если bindgen падает"
	@echo ""
	@echo "$(CYAN)## Сборка ##$(NC)"
	@echo "  $(GREEN)all$(NC)              Показать эту справку"
	@echo "  $(GREEN)build$(NC)            Собрать проект"
	@echo "  $(GREEN)build-whisper$(NC)    Собрать с whisper"
	@echo "  $(GREEN)build-both$(NC)       Собрать с vosk + whisper (переключение в audiosub.toml)"
	@echo "  $(GREEN)release$(NC)          Собрать в release-режиме (Linux)"
	@echo "  $(GREEN)release-whisper$(NC)  Release-сборка с whisper"
	@echo "  $(GREEN)release-both$(NC)    Release-сборка с vosk + whisper"
	@echo "  $(GREEN)release-win$(NC)      Кросс-сборка для Windows (нужен mingw-w64)"
	@echo "  $(GREEN)release-mac$(NC)      Собрать для macOS (запускать на macOS)"
	@echo ""
	@echo "---"
	@echo ""
	@echo "$(CYAN)## Запуск ##$(NC)"
	@echo "  $(GREEN)run [ARGS]$(NC)       Запустить с TUI (q/Esc для выхода)"
	@echo "  $(GREEN)run-whisper [ARGS]$(NC)  Запустить с whisper"
	@echo "  $(GREEN)run-both [ARGS]$(NC)  Запустить с vosk + whisper (выбор через audiosub.toml)"
	@echo "  $(GREEN)cli [ARGS]$(NC)       Запустить без TUI (консольный режим)"
	@echo "  $(GREEN)cli-whisper [ARGS]$(NC)  Консольный режим с whisper"
	@echo ""
	@echo "---"
	@echo ""
	@echo "$(CYAN)## Модели ##$(NC)"
	@echo "  $(GREEN)model-download$(NC)   Скачать модель Vosk (русская)"
	@echo "  $(GREEN)model-whisper$(NC)    Скачать модель Whisper ggml-base"
	@echo ""
	@echo "---"
	@echo ""
	@echo "$(CYAN)## Проверка качества ##$(NC)"
	@echo "  $(GREEN)test$(NC)             Запустить тесты"
	@echo "  $(GREEN)check$(NC)            cargo check (быстрая проверка)"
	@echo "  $(GREEN)lint$(NC)             cargo clippy"
	@echo "  $(GREEN)fmt$(NC)              Проверить форматирование"
	@echo "  $(GREEN)verify$(NC)           test + check + lint + fmt (CI-конвейер)"
	@echo "  $(GREEN)ci-check$(NC)         Полная CI-симуляция (scripts/ci-check.sh)"
	@echo ""
	@echo "---"
	@echo ""
	@echo "$(CYAN)## Docker ##$(NC)"
	@echo "  $(GREEN)docker$(NC)           Собрать + запустить через compose"
	@echo "  $(GREEN)docker-build$(NC)     Собрать Docker-образ"
	@echo "  $(GREEN)docker-run$(NC)       Запустить через docker compose"
	@echo ""
	@echo "---"
	@echo ""
	@echo "$(CYAN)## Утилиты ##$(NC)"
	@echo "  $(GREEN)clean$(NC)            Очистить артефакты сборки"
	@echo "  $(GREEN)report$(NC)           Создать шаблон отчёта"
	@echo "  $(GREEN)help$(NC)             Показать это сообщение"
	@echo ""
	@echo "$(YELLOW)Примеры:$(NC)"
	@echo "  make model-download"
	@echo "  make run"
	@echo "  make run-whisper"
	@echo "  make run ARGS='-- --list-devices'"
	@echo "  make cli"
	@echo "  make docker"

build:
	@echo "$(CYAN)→ Building $(APP_NAME)...$(NC)"
	cargo build
	@echo "$(GREEN)✓ Build complete$(NC)"

build-whisper:
	@echo "$(CYAN)→ Building $(APP_NAME) with whisper backend...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo build --no-default-features --features whisper,tui
	@echo "$(GREEN)✓ Build complete (whisper)$(NC)"

build-both:
	@echo "$(CYAN)→ Building $(APP_NAME) with vosk + whisper backends...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo build --features "vosk,whisper,tui"
	@echo "$(GREEN)✓ Build complete (vosk + whisper)$(NC)"
	@echo "$(YELLOW)  Set engine=\"vosk\" or engine=\"whisper\" in audiosub.toml to switch$(NC)"

release:
	@echo "$(CYAN)→ Building $(APP_NAME) (release)...$(NC)"
	cargo build --release
	mkdir -p release
	cp target/release/audiosub release/audiosub
	@echo "$(GREEN)✓ Release build complete — release/audiosub$(NC)"

release-whisper:
	@echo "$(CYAN)→ Building $(APP_NAME) (release, whisper backend)...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo build --release --no-default-features --features whisper,tui
	mkdir -p release
	cp target/release/audiosub release/audiosub-whisper
	@echo "$(GREEN)✓ Release build complete — release/audiosub-whisper$(NC)"

release-both:
	@echo "$(CYAN)→ Building $(APP_NAME) (release, vosk + whisper)...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo build --release --features "vosk,whisper,tui"
	mkdir -p release
	cp target/release/audiosub release/audiosub-both
	@echo "$(GREEN)✓ Release build complete — release/audiosub-both$(NC)"
	@echo "$(YELLOW)  Set engine=\"vosk\" or engine=\"whisper\" in audiosub.toml to switch$(NC)"

release-linux: release

release-win:
	@echo "$(CYAN)→ Building $(APP_NAME) for Windows...$(NC)"
	@if ! which x86_64-w64-mingw32-gcc >/dev/null 2>&1; then \
		echo "$(RED)✗ mingw-w64 not found. Install: sudo apt install mingw-w64$(NC)"; \
		exit 1; \
	fi
	rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
	cargo build --release --target x86_64-pc-windows-gnu
	mkdir -p release
	cp target/x86_64-pc-windows-gnu/release/audiosub.exe release/audiosub.exe
	@echo "$(GREEN)✓ Windows build complete — release/audiosub.exe$(NC)"

release-mac:
	@echo "$(RED)✗ macOS cross-compilation requires building natively on macOS.$(NC)"
	@echo "$(YELLOW)  On macOS: cargo build --release && cp target/release/audiosub release/audiosub.mac$(NC)"
	@exit 1

model-download:
	@echo "$(CYAN)→ Downloading $(VOSK_MODEL_NAME)...$(NC)"
	mkdir -p $(MODEL_DIR)
	curl -L -o /tmp/$(VOSK_MODEL_NAME).zip "$(VOSK_MODEL_URL)" && \
	unzip -qo /tmp/$(VOSK_MODEL_NAME).zip -d $(MODEL_DIR) && \
	rm /tmp/$(VOSK_MODEL_NAME).zip && \
	echo "$(GREEN)✓ Model downloaded to $(MODEL_DIR)/$(VOSK_MODEL_NAME)$(NC)" && \
	echo "$(YELLOW)  Edit audiosub.toml to change model path$(NC)"

model-whisper:
	@echo "$(CYAN)→ Downloading $(WHISPER_MODEL_NAME)...$(NC)"
	mkdir -p $(MODEL_DIR)
	curl -L -o $(MODEL_DIR)/$(WHISPER_MODEL_NAME) "$(WHISPER_MODEL_URL)" && \
	echo "$(GREEN)✓ Model downloaded to $(MODEL_DIR)/$(WHISPER_MODEL_NAME)$(NC)" && \
	echo "$(YELLOW)  Edit audiosub.toml to change model path$(NC)"

test:
	@echo "$(CYAN)→ Running tests...$(NC)"
	cargo test
	@echo "$(GREEN)✓ Tests passed$(NC)"

run:
	@echo "$(CYAN)→ Starting $(APP_NAME) (TUI mode)...$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib cargo run -- $(ARGS)

run-whisper:
	@echo "$(CYAN)→ Starting $(APP_NAME) (TUI, whisper backend)...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo run --no-default-features --features whisper,tui -- $(ARGS)

run-both:
	@echo "$(CYAN)→ Starting $(APP_NAME) (TUI, vosk + whisper)...$(NC)"
	@echo "$(YELLOW)  Switch engine in audiosub.toml: engine=\"vosk\" or engine=\"whisper\"$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib WHISPER_DONT_GENERATE_BINDINGS=1 cargo run --features "vosk,whisper,tui" -- $(ARGS)

cli:
	@echo "$(CYAN)→ Starting $(APP_NAME) (CLI mode)...$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib cargo run -- --no-tui $(ARGS)

cli-whisper:
	@echo "$(CYAN)→ Starting $(APP_NAME) (CLI, whisper backend)...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo run --no-default-features --features whisper,tui -- --no-tui $(ARGS)

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
