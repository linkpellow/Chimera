# chimera-core/Dockerfile
# The Rust "Body" - Stealth Browser Control
# Multi-stage build: Forge (compile) + Soldier (runtime)

# --- BUILD STAGE (The Forge) ---
FROM rust:1.76-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies for "reqwest-impersonate" (BoringSSL needs CMake/Clang)
# Also need protobuf-compiler for gRPC
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    clang \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests (if workspace exists)
# Handle both workspace and standalone builds
COPY Cargo.toml* ./
COPY chimera-core/Cargo.toml ./chimera-core/

# Create dummy main to force-build dependencies (Docker layer caching)
RUN mkdir -p chimera-core/src && \
    echo "fn main() {}" > chimera-core/src/main.rs && \
    cd chimera-core && \
    cargo build --release || true && \
    rm -rf src target/release/deps/chimera_core* target/release/chimera_core

# Copy source code, build script, and Proto
COPY chimera-core/src ./chimera-core/src
COPY chimera-core/build.rs ./chimera-core/
COPY proto ./proto

# Build the Apex Binary
RUN cd chimera-core && cargo build --release

# --- RUNTIME STAGE (The Soldier) ---
FROM debian:bookworm-slim

# 1. Install Chrome & Certificates (CRITICAL)
# We need actual Chromium for the Rust driver to control.
# Also need ca-certificates for HTTPS and libssl3 for reqwest-impersonate
RUN apt-get update && apt-get install -y \
    chromium \
    chromium-sandbox \
    ca-certificates \
    libssl3 \
    libnss3 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libdbus-1-3 \
    libxkbcommon0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libasound2 \
    libpango-1.0-0 \
    libcairo2 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 2. Copy the binary from the forge
COPY --from=builder /usr/src/app/chimera-core/target/release/chimera_core /app/chimera_core

# 3. Network & Config
ENV CHROME_BIN=/usr/bin/chromium
ENV CHIMERA_AGENT_ADDR=0.0.0.0:50051
ENV CHIMERA_VISION_ADDR=http://chimera-brain.railway.internal:50052
ENV CHIMERA_PROXY_PORT=8080
ENV RUST_LOG=info

# 4. Launch
EXPOSE 50051
CMD ["./chimera_core"]
