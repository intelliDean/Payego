## syntax=docker/dockerfile:1
#
#ARG RUST_VERSION=1.89.0
#ARG APP_NAME=payego
#
#################################################################################
## Create a stage for building the application.
#
#FROM rust:${RUST_VERSION}-slim-bullseye AS build
#ARG APP_NAME
#WORKDIR /app
#
## Fix GPG keys and install build dependencies
#RUN apt-get update && \
#    apt-get install -y --no-install-recommends \
#    ca-certificates \
#    && apt-get clean \
#    && rm -rf /var/lib/apt/lists/*
#
#RUN apt-get update && apt-get install -y \
#    libpq-dev \
#    pkg-config \
#    libssl-dev \
#    curl \
#    unzip \
#    make \
#    build-essential \
#    && rm -rf /var/lib/apt/lists/*
#
## Install Diesel CLI (this will be available in the build stage)
#RUN cargo install diesel_cli --no-default-features --features postgres
#
## Copy source code and dependency files
#COPY . .
#
## Build the application with cache mounts
#RUN --mount=type=cache,target=/app/target/ \
#    --mount=type=cache,target=/usr/local/cargo/git/db \
#    --mount=type=cache,target=/usr/local/cargo/registry/ \
#    cargo build --locked --release && \
#    cp ./target/release/$APP_NAME /bin/server
#
## Copy diesel binary to make it available in final stage
#RUN cp /usr/local/cargo/bin/diesel /bin/diesel
#
#################################################################################
## Create a new stage for running the application
#FROM debian:bullseye-slim AS final
#
## Install runtime dependencies
#RUN apt-get update && apt-get install -y \
#    libpq5 \
#    ca-certificates \
#    && rm -rf /var/lib/apt/lists/*
#
## Create a non-privileged user
#ARG UID=10001
#RUN adduser \
#    --disabled-password \
#    --gecos "" \
#    --home "/nonexistent" \
#    --shell "/sbin/nologin" \
#    --no-create-home \
#    --uid "${UID}" \
#    appuser
#
## Create app directory and set ownership
#RUN mkdir -p /app && chown appuser:appuser /app
#WORKDIR /app
#USER appuser
#
## Copy the executable, diesel cli, and migrations
#COPY --from=build --chown=appuser:appuser /bin/server /app/payego
#COPY --from=build --chown=appuser:appuser /bin/diesel /usr/local/bin/diesel
#COPY --from=build --chown=appuser:appuser /app/migrations /app/migrations
#
## Make diesel executable
#USER root
#RUN chmod +x /usr/local/bin/diesel
#USER appuser
#
## Expose the port that the application listens on.
#EXPOSE 8080
#
## Environment variables with defaults
#ENV RUST_LOG=info
#ENV HOST=0.0.0.0
#ENV PORT=8080
#
## What the container should run when it is started.
#CMD ["./payego"]

#==========

# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.89.0
ARG APP_NAME=payego

################################################################################
# Create a stage for building the application.
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    make \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Diesel CLI
RUN cargo install diesel_cli --no-default-features --features postgres --version 2.2.4

# Copy source code and dependency files
COPY . .

# Build the application with cache mounts
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

# Verify the binary exists
RUN ls -l /bin/server || echo "Binary not found!"

################################################################################
# Create a new stage for running the application
FROM debian:bullseye-slim AS final

# Install runtime dependencies and build tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    libpq-dev \
    libssl1.1 \
    ca-certificates \
    curl \
    build-essential \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Create a non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/home/appuser" \
    --shell "/bin/sh" \
    --uid "${UID}" \
    appuser

# Create app directory and set ownership
RUN mkdir -p /app && chown appuser:appuser /app
WORKDIR /app

# Install rustup and diesel_cli as appuser
USER appuser
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal && \
    /home/appuser/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres --version 2.2.4
ENV PATH="/home/appuser/.cargo/bin:${PATH}"

# Copy the executable and migrations
COPY --from=build --chown=appuser:appuser /bin/server /app/payego
COPY --from=build --chown=appuser:appuser /app/migrations /app/migrations

# Create a startup script with enhanced logging and retry logic
USER root
RUN echo '#!/bin/sh\n\
set -e\n\
echo "Using DATABASE_URL=$DATABASE_URL"\n\
echo "Waiting for database to be ready..."\n\
for i in $(seq 1 15); do\n\
  if psql "$DATABASE_URL" -c "\\q" 2>&1; then\n\
    echo "Database is ready!"\n\
    break\n\
  fi\n\
  echo "Database not ready, waiting 5 seconds... (attempt $i/15)"\n\
  sleep 5\n\
done\n\
if [ "$i" = 15 ]; then\n\
  echo "Error: Database not ready after 15 attempts"\n\
  exit 1\n\
fi\n\
echo "Running migrations with DATABASE_URL=$DATABASE_URL"\n\
/home/appuser/.cargo/bin/diesel migration run --database-url "$DATABASE_URL" 2>&1 || { echo "Migration failed"; exit 1; }\n\
echo "Starting payego application"\n\
exec /app/payego' > /app/start.sh && \
    chmod +x /app/start.sh && \
    chown appuser:appuser /app/start.sh

USER appuser

# Verify the binary exists
RUN ls -l /app/payego || echo "Binary not copied!"

# Expose the port
EXPOSE 8080

# Environment variables with defaults
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Run the startup script
CMD ["/app/start.sh"]