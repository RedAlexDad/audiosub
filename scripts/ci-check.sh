#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

echo -e "${BOLD}CI/CD Local Check${NC}"
echo "========================"
echo ""

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

FAILED=0

run_step "cargo check (no-default-features)" cargo check --no-default-features
FAILED=$((FAILED + $?))

run_step "cargo test (no-default-features)" cargo test --no-default-features
FAILED=$((FAILED + $?))

run_step "cargo clippy" cargo clippy --no-default-features -- -D warnings
FAILED=$((FAILED + $?))

run_step "cargo fmt --check" cargo fmt --check
FAILED=$((FAILED + $?))

echo "========================"
if [ "$FAILED" -eq 0 ]; then
    echo -e "${GREEN}${BOLD}All CI checks passed${NC}"
else
    echo -e "${RED}${BOLD}${FAILED} step(s) failed${NC}"
fi
exit "$FAILED"
