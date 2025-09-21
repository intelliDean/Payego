# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.89.0
ARG APP_NAME=payego

################################################################################
# Create a stage for building the application.

FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

# Fix GPG keys and install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    make \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Diesel CLI (this will be available in the build stage)
RUN cargo install diesel_cli --no-default-features --features postgres

# Copy source code and dependency files
COPY . .

# Build the application with cache mounts
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

# Copy diesel binary to make it available in final stage
RUN cp /usr/local/cargo/bin/diesel /bin/diesel

################################################################################
# Create a new stage for running the application
FROM debian:bullseye-slim AS final

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

# Create app directory and set ownership
RUN mkdir -p /app && chown appuser:appuser /app
WORKDIR /app
USER appuser

# Copy the executable, diesel cli, and migrations
COPY --from=build --chown=appuser:appuser /bin/server /app/payego
COPY --from=build --chown=appuser:appuser /bin/diesel /usr/local/bin/diesel
COPY --from=build --chown=appuser:appuser /app/migrations /app/migrations

# Make diesel executable
USER root
RUN chmod +x /usr/local/bin/diesel
USER appuser

# Expose the port that the application listens on.
EXPOSE 8080

# Environment variables with defaults
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# What the container should run when it is started.
CMD ["./payego"]