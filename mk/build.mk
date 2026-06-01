# ──────────────────────────────────────────────────────────────
# audiosub — сборка
# ──────────────────────────────────────────────────────────────

.PHONY: build

build:
	@echo "$(CYAN)→ Building $(APP_NAME)...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo build
	@echo "$(GREEN)✓ Build complete (whisper + vosk runtime)$(NC)"
