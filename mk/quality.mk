# ──────────────────────────────────────────────────────────────
# audiosub — проверка качества (check, lint, fmt, verify)
# ──────────────────────────────────────────────────────────────

.PHONY: check lint fmt verify ci-check

check:
	@echo "$(CYAN)→ Checking...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo check

lint:
	@echo "$(CYAN)→ Linting...$(NC)"
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo clippy -- -D warnings
	@echo "$(GREEN)✓ Lint passed$(NC)"

fmt:
	@echo "$(CYAN)→ Checking formatting...$(NC)"
	cargo fmt --check
	@echo "$(GREEN)✓ Formatting OK$(NC)"

verify:
	@echo "$(CYAN)══════════════════════════════════════════════$(NC)"; \
	echo "$(CYAN)  Verification pipeline$(NC)"; \
	echo "$(CYAN)══════════════════════════════════════════════$(NC)"; \
	echo ""; \
	echo "$(BOLD)[1/4] Running all tests...$(NC)"; \
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo test $(if $(filter 1,$(SHOW_DESCRIBE)),-- --show-output,) && \
	echo "$(GREEN)✓ Tests passed$(NC)" && \
	echo ""; \
	echo "$(BOLD)[2/4] Compilation check (cargo check)...$(NC)"; \
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo check && \
	echo "$(GREEN)✓ Check passed$(NC)" && \
	echo ""; \
	echo "$(BOLD)[3/4] Linting (cargo clippy)...$(NC)"; \
	WHISPER_DONT_GENERATE_BINDINGS=1 cargo clippy -- -D warnings && \
	echo "$(GREEN)✓ Lint passed$(NC)" && \
	echo ""; \
	echo "$(BOLD)[4/4] Formatting check (cargo fmt)...$(NC)"; \
	cargo fmt --check && \
	echo "$(GREEN)✓ Formatting OK$(NC)" && \
	echo ""; \
	echo "$(GREEN)══════════════════════════════════════════════$(NC)"; \
	echo "$(GREEN)✓ All checks passed$(NC)"

ci-check:
	@echo "$(CYAN)→ Running CI simulation...$(NC)"
	@scripts/ci-check.sh
