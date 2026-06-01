# ──────────────────────────────────────────────────────────────
# audiosub — запуск
# ──────────────────────────────────────────────────────────────

.PHONY: run cli

run:
	@echo "$(CYAN)→ Starting $(APP_NAME) (TUI)...$(NC)"
	@echo "$(YELLOW)  Engine: whisper (default), vosk (needs libvosk.so)$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib WHISPER_DONT_GENERATE_BINDINGS=1 cargo run -- $(ARGS)

cli:
	@echo "$(CYAN)→ Starting $(APP_NAME) (CLI mode)...$(NC)"
	LD_LIBRARY_PATH=/home/redalexdad/.local/lib WHISPER_DONT_GENERATE_BINDINGS=1 cargo run -- --no-tui $(ARGS)
