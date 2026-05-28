# ---- Build stage ----
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libpulse-dev libasound2-dev clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

COPY src/ src/
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpulse0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r audiosub && useradd -r -g audiosub -m -d /app audiosub
USER audiosub
WORKDIR /app

COPY --from=builder /app/target/release/audiosub /usr/local/bin/audiosub

ENTRYPOINT ["audiosub"]
