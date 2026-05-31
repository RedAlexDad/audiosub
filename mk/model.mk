# ──────────────────────────────────────────────────────────────
# audiosub — загрузка моделей
# ──────────────────────────────────────────────────────────────

.PHONY: model-download model-whisper

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
