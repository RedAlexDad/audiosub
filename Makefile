APP_NAME := audiosub
DOCKER_IMAGE := $(APP_NAME)
DOCKER_TAG := latest

.PHONY: all build test run lint clean docker docker-build docker-run fmt check report

all: build

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

run:
	cargo run -- $(ARGS)

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --check

check:
	cargo check

clean:
	cargo clean

docker-build:
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .

docker-run:
	docker compose up --build

docker:
	docker-build docker-run

report:
	@echo "Creating report..."
	@mkdir -p reports
	@echo "# Report $(shell date -u '+%Y-%m-%d %H:%M:%S UTC')" > reports/latest.md
	@echo "" >> reports/latest.md
	@echo "## Changes" >> reports/latest.md
	@echo "- " >> reports/latest.md
	@echo "" >> reports/latest.md
	@echo "## Problems" >> reports/latest.md
	@echo "- " >> reports/latest.md
	@echo "" >> reports/latest.md
	@echo "## Solutions" >> reports/latest.md
	@echo "- " >> reports/latest.md
	@echo "" >> reports/latest.md
	@echo "---" >> reports/latest.md
	@echo "Report generated automatically." >> reports/latest.md
	@echo "Created reports/latest.md — edit it before committing."
