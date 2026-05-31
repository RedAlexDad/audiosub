# ──────────────────────────────────────────────────────────────
# audiosub — Docker
# ──────────────────────────────────────────────────────────────

lib/vosk/libvosk.so:
	@mkdir -p lib/vosk
	cp /home/redalexdad/.local/lib/libvosk.so lib/vosk/libvosk.so

.PHONY: docker-build docker-run docker

docker-build: | lib/vosk/libvosk.so
	@echo "$(CYAN)→ Building Docker image (ENGINE=$(ENGINE))...$(NC)"
	ENGINE=$(ENGINE) DOCKER_BUILDKIT=0 docker build \
		--network host \
		--build-arg ENGINE=$(ENGINE) \
		-t $(DOCKER_IMAGE):$(ENGINE) .
	@echo "$(GREEN)✓ Docker image built: $(DOCKER_IMAGE):$(ENGINE)$(NC)"
	@echo "$(YELLOW)  ENGINE=$(ENGINE) — set ENGINE=whisper or ENGINE=both for other backends$(NC)"

docker-run:
	@echo "$(CYAN)→ Starting Docker Compose...$(NC)"
	ENGINE=$(ENGINE) USER_ID=$(shell id -u) GROUP_ID=$(shell id -g) docker compose up

docker: docker-run
