# ──────────────────────────────────────────────────────────────
# audiosub — сборка
# ──────────────────────────────────────────────────────────────

.PHONY: build build-whisper build-both

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
