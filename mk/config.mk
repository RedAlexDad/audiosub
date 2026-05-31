# ──────────────────────────────────────────────────────────────
# audiosub — общие переменные и цвета
# ──────────────────────────────────────────────────────────────

APP_NAME       := audiosub
DOCKER_IMAGE   := $(APP_NAME)
DOCKER_TAG     := latest

# --- Vosk ---
VOSK_MODEL_URL  := https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip
VOSK_MODEL_NAME := vosk-model-small-ru-0.22

# --- Whisper ---
WHISPER_MODEL_URL  := https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
WHISPER_MODEL_NAME := ggml-base.bin

MODEL_DIR := models
MODEL_PATH := $(MODEL_DIR)/$(VOSK_MODEL_NAME)

GREEN  := \033[0;32m
CYAN   := \033[0;36m
YELLOW := \033[1;33m
RED    := \033[0;31m
BOLD   := \033[1m
NC     := \033[0m

# Resolve model path to absolute — Vosk needs absolute path
AUDIOSUB_MODEL := $(CURDIR)/$(MODEL_DIR)/$(MODEL_NAME)

# Docker engine selection: vosk, whisper, both
ENGINE ?= vosk

# Test output: 1 = show descriptions (--show-output), 0 = compact
SHOW_DESCRIBE ?= 0
