# Makefile for ahlan-commerce onboarding project

# Detect OS to determine the correct Atlas executable path
ATLAS ?= atlas
ifeq ($(OS),Windows_NT)
    ATLAS := ./atlas.exe
endif

DB_HOST ?= localhost
DB_PORT ?= 5432
DB_USER ?= postgres
DB_NAME ?= ahlan_commerce
DATABASE_URL ?= postgres://$(DB_USER)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)
PSQL ?= psql
CREATEDB ?= createdb
PG_ISREADY ?= pg_isready
PG_CTL ?= pg_ctl
PG_SERVICE ?= postgresql-x64-16
PGDATA ?= E:/Set_up_Porgrams/PostgreSql/data

.PHONY: start stop run-api run-admin test db-start db-create db-check db-migrate health cornucopia-generate redis-health

start:
	mprocs -c mprocs.yaml

stop:
	@echo "Using local PostgreSQL. Stop it with your local PostgreSQL service manager if needed."

run-api:
	cargo run -p api

run-admin:
	cd apps/admin && npm run dev

test:
	cargo test -- --test-threads=1

db-start:
ifeq ($(OS),Windows_NT)
	powershell -NoProfile -Command "$$service = Get-Service -Name '$(PG_SERVICE)' -ErrorAction SilentlyContinue; if ($$service -and $$service.Status -ne 'Running') { try { Start-Service -Name '$(PG_SERVICE)' -ErrorAction Stop } catch { Write-Warning $$_.Exception.Message } }"
	$(PG_ISREADY) -h $(DB_HOST) -p $(DB_PORT) -U $(DB_USER) || $(PG_CTL) -D "$(PGDATA)" -l "$(PGDATA)/postgresql-local.log" start
endif
	$(PG_ISREADY) -h $(DB_HOST) -p $(DB_PORT) -U $(DB_USER)

db-create:
ifeq ($(OS),Windows_NT)
	$(PSQL) -h $(DB_HOST) -p $(DB_PORT) -U $(DB_USER) -d postgres -tc "SELECT 1 FROM pg_database WHERE datname = '$(DB_NAME)'" | findstr 1 >NUL || $(CREATEDB) -h $(DB_HOST) -p $(DB_PORT) -U $(DB_USER) $(DB_NAME)
else
	$(PSQL) -h $(DB_HOST) -p $(DB_PORT) -U $(DB_USER) -d postgres -tc "SELECT 1 FROM pg_database WHERE datname = '$(DB_NAME)'" | grep -q 1 || $(CREATEDB) -h $(DB_HOST) -p $(DB_PORT) -U $(DB_USER) $(DB_NAME)
endif

db-check: db-start db-create

db-migrate:
	$(ATLAS) migrate apply --env local

health:
	curl -f http://localhost:3000/health

redis-health:
	redis-cli ping

cornucopia-generate:
ifeq ($(OS),Windows_NT)
	powershell -NoProfile -Command "$$root = (Get-Location).Path; $$tmp = \"$$root\target\tmp\"; New-Item -ItemType Directory -Force -Path $$tmp | Out-Null; $$env:TEMP = $$tmp; $$env:TMP = $$tmp; cornucopia live '$(DATABASE_URL)' -q 'db/queries' -d 'packages/db' --async true"
else
	cornucopia live "$(DATABASE_URL)" -q "db/queries" -d "packages/db" --async true
endif