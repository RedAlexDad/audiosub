# ──────────────────────────────────────────────────────────────
# audiosub — утилиты (clean, report)
# ──────────────────────────────────────────────────────────────

.PHONY: clean report

clean:
	@echo "$(YELLOW)← Cleaning...$(NC)"
	cargo clean
	@echo "$(GREEN)✓ Cleaned$(NC)"

report:
	@echo "$(CYAN)→ Creating report...$(NC)"
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
	@echo "$(GREEN)✓ Created reports/latest.md$(NC) — edit it before committing."
