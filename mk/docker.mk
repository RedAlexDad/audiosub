# ──────────────────────────────────────────────────────────────
# audiosub — Docker (docker compose v2 only)
# ──────────────────────────────────────────────────────────────

ENGINE ?= vosk
COMPOSE ?= docker compose
DOCKER_IMAGE ?= audiosub

# ── Build ────────────────────────────────────────────────────

docker-build:
	@echo "$(CYAN)→ Build image (ENGINE=$(ENGINE))...$(NC)"
	ENGINE=$(ENGINE) docker build \
		--build-arg ENGINE=$(ENGINE) \
		-t $(DOCKER_IMAGE):$(ENGINE) .
	@echo "$(GREEN)✓ Image: $(DOCKER_IMAGE):$(ENGINE) (whisper + $(YELLOW)vosk runtime$(GREEN))$(NC)"

docker-rebuild:
	@echo "$(CYAN)→ Rebuild (no cache, ENGINE=$(ENGINE))...$(NC)"
	ENGINE=$(ENGINE) DOCKER_BUILDKIT=0 docker build \
		--network host --no-cache \
		--build-arg ENGINE=$(ENGINE) \
		-t $(DOCKER_IMAGE):$(ENGINE) .
	@echo "$(GREEN)✓ Rebuilt: $(DOCKER_IMAGE):$(ENGINE)$(NC)"

# ── Lifecycle ────────────────────────────────────────────────

docker-up:
	@echo "$(CYAN)→ Up...$(NC)"
	ENGINE=$(ENGINE) USER_ID=$(shell id -u) GROUP_ID=$(shell id -g) \
		$(COMPOSE) up
	@echo "$(GREEN)✓ Stopped$(NC)"

docker-down:
	@echo "$(CYAN)→ Down...$(NC)"
	$(COMPOSE) down
	@echo "$(GREEN)✓ Stopped$(NC)"

docker-deploy: docker-build docker-up

docker-restart: docker-down docker-up

docker-clean:
	@echo "$(CYAN)→ Remove image $(DOCKER_IMAGE):$(ENGINE)...$(NC)"
	-$(COMPOSE) down --rmi all 2>/dev/null
	-docker rmi $(DOCKER_IMAGE):$(ENGINE) 2>/dev/null
	@echo "$(GREEN)✓ Cleaned$(NC)"

# ── TUI mode ────────────────────────────────────────────────

docker-run: docker-build
	@echo "$(CYAN)→ Starting TUI...$(NC)"
	ENGINE=$(ENGINE) USER_ID=$(shell id -u) GROUP_ID=$(shell id -g) \
		$(COMPOSE) run --rm --service-ports \
		audiosub

# ── Logs ─────────────────────────────────────────────────────

docker-logs:
	$(COMPOSE) logs -f

# ── Default ──────────────────────────────────────────────────

docker: docker-deploy
