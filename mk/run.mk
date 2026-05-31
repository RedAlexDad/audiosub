# ──────────────────────────────────────────────────────────────
# audiosub — запуск
# ──────────────────────────────────────────────────────────────

.PHONY: run run-whisper run-both cli cli-whisper

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
