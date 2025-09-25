# Multi-stage build for Rust server
# Build stage
FROM rust:1.75-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml ./

# Create a dummy src directory and main.rs for dependency building
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# Build dependencies only (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release && rm src/main.rs

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage - use a minimal base image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create application directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/rust-selfhost-server /usr/local/bin/rust-selfhost-server

# Create a non-root user for running the application
RUN useradd -r -u 1000 appuser && \
    chown appuser:appuser /usr/local/bin/rust-selfhost-server

# Switch to non-root user
USER appuser

# Expose the application port
EXPOSE 3000

# Set a sensible default log level
ENV RUST_LOG=info

# Run the application
ENTRYPOINT ["rust-selfhost-server"]
