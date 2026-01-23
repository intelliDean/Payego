# syntax=docker/dockerfile:1

# ------------------------------------------------------------------------------
# 1. Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.81-slim-bullseye AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
  libpq-dev \
  pkg-config \
  libssl-dev \
  build-essential \
  && rm -rf /var/lib/apt/lists/*

# Install Diesel CLI (pre-built for the final stage)
RUN cargo install diesel_cli --no-default-features --features postgres --version 2.2.4

# Copy only the dependency manifest files first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./
COPY bin/payego/Cargo.toml bin/payego/
COPY crates/api/Cargo.toml crates/api/
COPY crates/core/Cargo.toml crates/core/
COPY crates/primitives/Cargo.toml crates/primitives/

# Create dummy source files to build dependencies
RUN mkdir -p bin/payego/src crates/api/src crates/core/src crates/primitives/src && \
  echo "fn main() {}" > bin/payego/src/main.rs && \
  echo "pub fn dummy() {}" > crates/api/src/lib.rs && \
  echo "pub fn dummy() {}" > crates/core/src/lib.rs && \
  echo "pub fn dummy() {}" > crates/primitives/src/lib.rs

# Pre-build dependencies
RUN cargo build --release --locked

# Now copy the actual source code and build the real binary
COPY . .
RUN cargo build --release --locked --bin payego

# ------------------------------------------------------------------------------
# 2. Runtime Stage
# ------------------------------------------------------------------------------
FROM debian:bullseye-slim AS final

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
  libpq5 \
  ca-certificates \
  postgresql-client \
  && rm -rf /var/lib/apt/lists/*

# Create a non-privileged user
RUN adduser --disabled-password --gecos "" --home "/app" --shell "/bin/sh" --uid 10001 appuser

WORKDIR /app

# Copy binaries and migrations
COPY --from=builder /app/target/release/payego /app/payego
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /app/migrations /app/migrations

# Add a startup script for migrations and app launch
COPY <<EOF /app/start.sh
#!/bin/sh
set -e

echo "Waiting for database to be ready..."
until pg_isready -h "\$(echo \$DATABASE_URL | sed -e 's/.*@//' -e 's/:.*//')" -p "\$(echo \$DATABASE_URL | sed -e 's/.*://' -e 's/\/.*//')"; do
  echo "Database not ready, waiting..."
  sleep 2
done

echo "Running migrations..."
diesel migration run --database-url "\$DATABASE_URL"

echo "Starting Payego..."
exec /app/payego
EOF

RUN chmod +x /app/start.sh && chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

CMD ["/app/start.sh"]