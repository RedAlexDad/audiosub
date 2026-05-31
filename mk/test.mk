# ──────────────────────────────────────────────────────────────
# audiosub — тесты
# ──────────────────────────────────────────────────────────────

.PHONY: test

test:
	@echo "$(CYAN)→ Running tests...$(NC)"
	cargo test $(if $(filter 1,$(SHOW_DESCRIBE)),-- --show-output,)
	@echo "$(GREEN)✓ Tests passed$(NC)"
