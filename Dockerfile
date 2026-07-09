FROM rust:1.96-slim AS builder

# Set working directory
WORKDIR /usr/src/ahlan-commerce

# Install dependencies needed for compiling certain Rust crates (like OpenSSL and Postgres drivers)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Limit parallel compile jobs to avoid OOM on small servers.
# 2 jobs keeps peak RAM around 3-4 GB. Disable incremental to save disk I/O.
ENV CARGO_BUILD_JOBS=2
ENV CARGO_INCREMENTAL=0

# Copy all the source code
COPY . .

# Build both API and Worker binaries in release mode
RUN cargo build --release -p api -p worker

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
