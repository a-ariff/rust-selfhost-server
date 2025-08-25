# Multi-stage build for Rust server
# Build stage
FROM rust:1-bookworm AS builder

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached if Cargo.toml/Cargo.lock don't change)
RUN cargo build --release

# Remove the dummy main.rs
RUN rm src/main.rs

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12 AS runtime

# Install ca-certificates for HTTPS requests
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rust-selfhost-server /usr/local/bin/rust-selfhost-server

# Expose port
EXPOSE 3000

# Set the startup command
CMD ["rust-selfhost-server"]
