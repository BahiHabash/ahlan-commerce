# Makefile for ahlan-commerce onboarding project

# Detect OS to determine the correct Atlas executable path
ATLAS ?= atlas
ifeq ($(OS),Windows_NT)
    ATLAS := ./atlas.exe
endif

.PHONY: run-api test db-start db-migrate health

run-api:
	cargo run -p api

test:
	cargo test -- --test-threads=1

db-start:
	docker start catalog-db || docker run --name catalog-db -p 5432:5432 -e POSTGRES_HOST_AUTH_METHOD=trust -d postgres

db-migrate:
	$(ATLAS) migrate apply --env local

health:
	curl -f http://localhost:3000/health
