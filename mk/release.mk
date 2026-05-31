# ──────────────────────────────────────────────────────────────
# audiosub — release-сборки
# ──────────────────────────────────────────────────────────────

.PHONY: release release-whisper release-both release-linux release-win release-mac

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
