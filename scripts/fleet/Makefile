# Makefile for LLMSpell Fleet Management
# Simple commands for managing multiple kernel processes

FLEET_DIR := $(HOME)/.llmspell/fleet
FLEET_SCRIPT := ./llmspell-fleet
PYTHON_FLEET := python3 fleet_manager.py

# Default configuration files
CONFIG_OPENAI ?= ../configs/openai.toml
CONFIG_ANTHROPIC ?= ../configs/anthropic.toml
CONFIG_LOCAL ?= ../configs/local.toml

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
NC := \033[0m # No Color

.PHONY: help init spawn-openai spawn-anthropic spawn-local list stop-all cleanup health docker-up docker-down test

help:
	@echo "$(GREEN)LLMSpell Fleet Management$(NC)"
	@echo "========================="
	@echo ""
	@echo "$(YELLOW)Basic Commands:$(NC)"
	@echo "  make init          - Initialize fleet directory"
	@echo "  make spawn         - Spawn default kernel"
	@echo "  make list          - List running kernels"
	@echo "  make stop-all      - Stop all kernels"
	@echo "  make cleanup       - Clean up dead kernels"
	@echo "  make health        - Check kernel health"
	@echo ""
	@echo "$(YELLOW)Spawn Specific Kernels:$(NC)"
	@echo "  make spawn-openai     - Spawn with OpenAI config"
	@echo "  make spawn-anthropic  - Spawn with Anthropic config"
	@echo "  make spawn-local      - Spawn with local config"
	@echo ""
	@echo "$(YELLOW)Docker Commands:$(NC)"
	@echo "  make docker-build  - Build Docker image"
	@echo "  make docker-up     - Start Docker fleet"
	@echo "  make docker-down   - Stop Docker fleet"
	@echo "  make docker-logs   - View Docker logs"
	@echo ""
	@echo "$(YELLOW)Development:$(NC)"
	@echo "  make test          - Run fleet tests"
	@echo "  make demo          - Run demo scenario"
	@echo "  make metrics       - Show fleet metrics"

# Initialize fleet directory structure
init:
	@echo "$(GREEN)Initializing fleet directory...$(NC)"
	@mkdir -p $(FLEET_DIR)
	@mkdir -p $(FLEET_DIR)/logs
	@mkdir -p configs
	@chmod +x $(FLEET_SCRIPT)
	@echo "$(GREEN)✓ Fleet directory initialized at $(FLEET_DIR)$(NC)"

# Spawn kernels with different configurations
spawn:
	@echo "$(GREEN)Spawning default kernel...$(NC)"
	@$(FLEET_SCRIPT) spawn

spawn-openai: init
	@echo "$(GREEN)Spawning OpenAI kernel...$(NC)"
	@$(FLEET_SCRIPT) spawn $(CONFIG_OPENAI) lua

spawn-anthropic: init
	@echo "$(GREEN)Spawning Anthropic kernel...$(NC)"
	@$(FLEET_SCRIPT) spawn $(CONFIG_ANTHROPIC) lua

spawn-local: init
	@echo "$(GREEN)Spawning local kernel...$(NC)"
	@$(FLEET_SCRIPT) spawn $(CONFIG_LOCAL) lua

# List all running kernels
list:
	@$(FLEET_SCRIPT) list

# Stop all kernels
stop-all:
	@echo "$(RED)Stopping all kernels...$(NC)"
	@$(FLEET_SCRIPT) stop-all

# Clean up dead kernels
cleanup:
	@echo "$(YELLOW)Cleaning up dead kernels...$(NC)"
	@$(FLEET_SCRIPT) cleanup

# Health check
health:
	@$(FLEET_SCRIPT) health

# Docker commands
docker-build:
	@echo "$(GREEN)Building Docker image...$(NC)"
	@docker build -f Dockerfile -t llmspell:latest ../..

docker-up: docker-build
	@echo "$(GREEN)Starting Docker fleet...$(NC)"
	@docker-compose up -d

docker-down:
	@echo "$(RED)Stopping Docker fleet...$(NC)"
	@docker-compose down

docker-logs:
	@docker-compose logs -f

docker-ps:
	@docker-compose ps

# Python fleet manager
python-spawn:
	@$(PYTHON_FLEET) spawn

python-list:
	@$(PYTHON_FLEET) list --verbose

python-metrics:
	@$(PYTHON_FLEET) metrics

# Demo scenario
demo: init
	@echo "$(GREEN)Running fleet demo...$(NC)"
	@echo ""
	@echo "1. Spawning kernel for Developer A (OpenAI)..."
	@$(FLEET_SCRIPT) spawn $(CONFIG_OPENAI) lua
	@sleep 2
	@echo ""
	@echo "2. Spawning kernel for Developer B (Anthropic)..."
	@$(FLEET_SCRIPT) spawn $(CONFIG_ANTHROPIC) lua
	@sleep 2
	@echo ""
	@echo "3. Listing all kernels..."
	@$(FLEET_SCRIPT) list
	@echo ""
	@echo "$(GREEN)✓ Demo complete! Two isolated kernels running.$(NC)"
	@echo "$(YELLOW)Connect to them using the connection files shown above.$(NC)"

# Testing
test:
	@echo "$(GREEN)Running fleet tests...$(NC)"
	@./test_fleet.sh

# Metrics
metrics:
	@echo "$(GREEN)Fleet Metrics:$(NC)"
	@echo "────────────────"
	@echo -n "Total Kernels: "
	@$(FLEET_SCRIPT) list | grep "kernel-" | wc -l
	@echo -n "Total Memory: "
	@ps aux | grep "llmspell kernel" | awk '{sum+=$6} END {print int(sum/1024) "MB"}'
	@echo -n "Total CPU: "
	@ps aux | grep "llmspell kernel" | awk '{sum+=$3} END {print sum "%"}'

# Install fleet commands globally
install:
	@echo "$(GREEN)Installing fleet commands...$(NC)"
	@sudo ln -sf $(PWD)/$(FLEET_SCRIPT) /usr/local/bin/llmspell-fleet
	@sudo ln -sf $(PWD)/fleet_manager.py /usr/local/bin/llmspell-fleet-python
	@echo "$(GREEN)✓ Fleet commands installed$(NC)"
	@echo "  - llmspell-fleet (shell)"
	@echo "  - llmspell-fleet-python (Python)"

# Uninstall
uninstall:
	@echo "$(RED)Removing fleet commands...$(NC)"
	@sudo rm -f /usr/local/bin/llmspell-fleet
	@sudo rm -f /usr/local/bin/llmspell-fleet-python

# Clean everything
clean: stop-all
	@echo "$(RED)Cleaning fleet directory...$(NC)"
	@rm -rf $(FLEET_DIR)
	@echo "$(GREEN)✓ Fleet directory cleaned$(NC)"

# Status summary
status: list health metrics

# Quick start
quickstart: init spawn-openai spawn-anthropic list
	@echo ""
	@echo "$(GREEN)✓ Quick start complete!$(NC)"
	@echo "$(YELLOW)Two kernels running with different configs.$(NC)"
	@echo "$(YELLOW)Use 'make list' to see them.$(NC)"

.DEFAULT_GOAL := help