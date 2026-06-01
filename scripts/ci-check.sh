#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────
# Локальный CI-checkscript — дублирует логику .github/workflows/ci.yml
# Запускать из корня проекта:  make ci-check
# ──────────────────────────────────────────────────────────────
set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

echo -e "${BOLD}CI/CD Local Check${NC}"
echo "========================"
echo ""

FAILED=0

run_step() {
    local name="$1"
    shift
    echo -e "${CYAN}→ $name...${NC}"
    if "$@" 2>&1; then
        echo -e "${GREEN}✓ $name passed${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ $name FAILED${NC}"
        echo ""
        return 1
    fi
}

# ── Format ──────────────────────────────────
run_step "cargo fmt --check" cargo fmt --check
FAILED=$((FAILED + $?))

# ── Clippy (matrix) ─────────────────────────
run_step "clippy (minimal)" cargo clippy --no-default-features -- -D warnings
FAILED=$((FAILED + $?))

run_step "clippy (default)" cargo clippy -- -D warnings
FAILED=$((FAILED + $?))

run_step "clippy (whisper+tui)" cargo clippy --no-default-features --features whisper,tui -- -D warnings
FAILED=$((FAILED + $?))

# ── Check (matrix) ──────────────────────────
run_step "check (minimal)" cargo check --no-default-features
FAILED=$((FAILED + $?))

run_step "check (default)" cargo check
FAILED=$((FAILED + $?))

run_step "check (whisper+tui)" cargo check --no-default-features --features whisper,tui
FAILED=$((FAILED + $?))

# ── Tests ──────────────────────────────────
run_step "cargo test" cargo test --no-default-features
FAILED=$((FAILED + $?))

echo "========================"
if [ "$FAILED" -eq 0 ]; then
    echo -e "${GREEN}${BOLD}All CI checks passed${NC}"
else
    echo -e "${RED}${BOLD}${FAILED} step(s) failed${NC}"
fi
exit "$FAILED"
