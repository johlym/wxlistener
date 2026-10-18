# Multi-stage Dockerfile for wxlistener
# Stage 1: Build the Rust binary
FROM rust:1.91-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Build for release
RUN cargo build --release

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 wxlistener

# Copy binary from builder
COPY --from=builder /app/target/release/wxlistener /usr/local/bin/wxlistener

# Set ownership
RUN chown wxlistener:wxlistener /usr/local/bin/wxlistener

# Switch to non-root user
USER wxlistener

# Create config directory
WORKDIR /home/wxlistener

# Environment variables for configuration
ENV WXLISTENER_IP="" \
    WXLISTENER_PORT="80" \
    WXLISTENER_LEGACY="" \
    WXLISTENER_INTERVAL="60" \
    WXLISTENER_FORMAT="text"

# Health check
HEALTHCHECK --interval=120s --timeout=10s --start-period=10s --retries=3 \
    CMD wxlistener --ip ${WXLISTENER_IP} --port ${WXLISTENER_PORT} || exit 1

# Default command: continuous mode
ENTRYPOINT ["sh", "-c"]
CMD ["legacy=; case \"$WXLISTENER_LEGACY\" in 1|true|TRUE|yes|YES|on|ON) legacy=--legacy;; esac; wxlistener --ip ${WXLISTENER_IP} $legacy --port ${WXLISTENER_PORT} --continuous ${WXLISTENER_INTERVAL} --format ${WXLISTENER_FORMAT}"]

# Labels
LABEL org.opencontainers.image.title="wxlistener"
LABEL org.opencontainers.image.description="GW1000/Ecowitt Gateway Weather Station Listener"
LABEL org.opencontainers.image.version="0.5.1"
LABEL org.opencontainers.image.authors="Johnathan Lyman"
