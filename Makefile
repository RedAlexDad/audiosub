APP_NAME := audiosub
DOCKER_IMAGE := $(APP_NAME)
DOCKER_TAG := latest

GREEN  := \033[0;32m
CYAN   := \033[0;36m
YELLOW := \033[1;33m
RED    := \033[0;31m
BOLD   := \033[1m
NC     := \033[0m

.PHONY: all build test run lint clean docker docker-build docker-run fmt check verify report help

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
	@echo "  $(GREEN)all$(NC)           Show this help"
	@echo "  $(GREEN)build$(NC)         Compile the project"
	@echo "  $(GREEN)release$(NC)       Compile in release mode"
	@echo "  $(GREEN)run [ARGS]$(NC)    Run with arguments (e.g. make run ARGS='-- --help')"
	@echo "  $(GREEN)test$(NC)          Run tests"
	@echo "  $(GREEN)check$(NC)         cargo check (fast)"
	@echo "  $(GREEN)lint$(NC)          cargo clippy"
	@echo "  $(GREEN)fmt$(NC)           Check formatting"
	@echo "  $(GREEN)verify$(NC)        Run test + check + lint + fmt (CI pipeline)"
	@echo "  $(GREEN)clean$(NC)         Remove build artifacts"
	@echo "  $(GREEN)docker$(NC)        Build + run via compose"
	@echo "  $(GREEN)docker-build$(NC)  Build Docker image"
	@echo "  $(GREEN)docker-run$(NC)    Run via docker compose"
	@echo "  $(GREEN)report$(NC)        Create report template"
	@echo "  $(GREEN)help$(NC)          Show this message"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make run ARGS='-- --engine vosk --output subs.srt'"
	@echo "  make docker"

build:
	@echo "$(CYAN)→ Building $(APP_NAME)...$(NC)"
	cargo build
	@echo "$(GREEN)✓ Build complete$(NC)"

release:
	@echo "$(CYAN)→ Building $(APP_NAME) (release)...$(NC)"
	cargo build --release
	@echo "$(GREEN)✓ Release build complete$(NC)"

test:
	@echo "$(CYAN)→ Running tests...$(NC)"
	cargo test
	@echo "$(GREEN)✓ Tests passed$(NC)"

run:
	@echo "$(CYAN)→ Starting $(APP_NAME)...$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib cargo run -- $(ARGS)

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

clean:
	@echo "$(YELLOW)← Cleaning...$(NC)"
	cargo clean
	@echo "$(GREEN)✓ Cleaned$(NC)"

docker-build:
	@echo "$(CYAN)→ Building Docker image...$(NC)"
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .
	@echo "$(GREEN)✓ Docker image built$(NC)"

docker-run:
	@echo "$(CYAN)→ Starting Docker Compose...$(NC)"
	docker compose up --build

docker: docker-build docker-run

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
