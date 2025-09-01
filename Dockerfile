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
COPY Cargo.toml Cargo.lock ./

# Create a dummy src directory and main.rs for dependency building
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# Build dependencies only (this layer will be cached unless Cargo.toml/Cargo.lock changes)
RUN cargo build --release --locked && rm src/main.rs

# Copy source code
COPY src ./src

# Build the application with locked dependencies
RUN cargo build --release --locked

# Runtime stage - use a minimal base image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rust-selfhost-server /usr/local/bin/rust-selfhost-server

# Set proper permissions
RUN chmod +x /usr/local/bin/rust-selfhost-server

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Set the startup command
CMD ["rust-selfhost-server"]
