# Multi-stage Dockerfile for LLMSpell kernel fleet deployment
# Optimized for size, security, and fleet orchestration

# Build stage
FROM rust:1.81-bookworm as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy project files from context (build context is repo root via Makefile)
COPY . .

# Build with release optimizations
RUN cargo build --release --features "lua" && \
    strip target/release/llmspell

# Runtime stage - minimal image for fleet deployment
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    netcat-openbsd \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user and directories
RUN useradd -m -u 1000 -s /bin/bash llmspell && \
    mkdir -p /var/lib/llmspell/connections \
             /var/log/llmspell \
             /etc/llmspell \
             /workspace && \
    chown -R llmspell:llmspell /var/lib/llmspell /var/log/llmspell /etc/llmspell /workspace

# Copy binary from builder
COPY --from=builder /build/target/release/llmspell /usr/local/bin/llmspell

# Set user
USER llmspell
WORKDIR /workspace

# Default environment variables
ENV LLMSPELL_CONNECTION_FILE=/var/lib/llmspell/connections/kernel.json \
    LLMSPELL_LOG_FILE=/var/log/llmspell/kernel.log \
    LLMSPELL_PID_FILE=/var/run/kernel.pid \
    KERNEL_PORT=9555

# Expose default kernel port range for fleet
EXPOSE 9555-9600

# Health check using netcat
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD nc -z localhost ${KERNEL_PORT} || exit 1

# Default command - kernel start with daemon mode
ENTRYPOINT ["llmspell", "kernel", "start"]
CMD ["--daemon", "--port", "9555", "--connection-file", "/var/lib/llmspell/connections/kernel.json", "--log-file", "/var/log/llmspell/kernel.log"]