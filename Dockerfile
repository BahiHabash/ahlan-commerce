FROM rust:1.96-slim AS builder

# Set working directory
WORKDIR /usr/src/ahlan-commerce

# Install dependencies needed for compiling certain Rust crates (like OpenSSL and Postgres drivers)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Limit parallel compile jobs to avoid OOM on small servers (2 jobs ~= 3-4 GB RAM peak)
ENV CARGO_BUILD_JOBS=2

# Copy manifests first to cache dependencies as a separate Docker layer.
# This means re-builds after code changes won't recompile all crates from scratch.
COPY Cargo.toml Cargo.lock ./
COPY apps/api/Cargo.toml apps/api/Cargo.toml
COPY apps/worker/Cargo.toml apps/worker/Cargo.toml
COPY packages/catalog/Cargo.toml packages/catalog/Cargo.toml
COPY packages/cache/Cargo.toml packages/cache/Cargo.toml
COPY packages/db/Cargo.toml packages/db/Cargo.toml

# Create dummy source files so cargo can compile all dependencies
RUN mkdir -p apps/api/src apps/worker/src packages/catalog/src packages/cache/src packages/db/src \
    && echo "fn main() {}" > apps/api/src/main.rs \
    && echo "fn main() {}" > apps/worker/src/main.rs \
    && echo "" > packages/catalog/src/lib.rs \
    && echo "" > packages/cache/src/lib.rs \
    && echo "" > packages/db/src/lib.rs \
    && cargo build --release -p api -p worker \
    && rm -rf apps/api/src apps/worker/src packages/catalog/src packages/cache/src packages/db/src

# Copy all the source code
COPY . .

# Touch the main files to bust the cached binary and rebuild only our code
RUN touch apps/api/src/main.rs apps/worker/src/main.rs \
    && cargo build --release -p api -p worker

FROM debian:bookworm-slim AS runtime

# Install CA certificates and curl
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Install Atlas CLI
RUN curl -sSf https://atlasgo.sh | sh

WORKDIR /app

# Copy migration files for pre-deploy scripts
COPY atlas.hcl .
COPY db/migrations db/migrations

# Copy the built binaries from the builder stage
COPY --from=builder /usr/src/ahlan-commerce/target/release/api /usr/local/bin/api
COPY --from=builder /usr/src/ahlan-commerce/target/release/worker /usr/local/bin/worker

# Expose the API port assuming the standard 3000
EXPOSE 3000

# The startup command is defaulted to the API
CMD ["api"]
