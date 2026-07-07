# Setup Guide

This document outlines the steps required to get the Ahlan Commerce project running locally on your machine.

## Prerequisites

- **Rust**: Ensure you have a recent version of the Rust toolchain installed (use `rustup`).
- **PostgreSQL**: A running instance of PostgreSQL is required (version 16 is recommended).
- **Make**: Used to run standard commands.
- **Node.js**: Required if you plan to run the Admin frontend locally.
- **Atlas**: Database migration tool.

## Initializing the Database

1. Ensure the PostgreSQL service is running. If you have the standard configuration in the Makefile, you can start the service with:
   ```bash
   make db-start
   ```
2. Create the database (`ahlan_commerce`):
   ```bash
   make db-create
   ```
3. Apply migrations to the database using Atlas:
   ```bash
   make db-migrate
   ```
   *(Alternatively, run `make db-check` which runs start and create).*

## Running the Application

To run the API server locally:
```bash
make run-api
```
This will start the Axum web server on port 3000. You can check the health endpoint at `http://localhost:3000/health`.

To run the full stack including standard background processes (using `mprocs`):
```bash
make start
```
