ARG ENGINE=vosk

# ============================================================
# Builder: compile audiosub with selected engine(s)
# ============================================================
FROM rust:latest AS builder
ARG ENGINE
WORKDIR /app

RUN apt-get update && apt-get install -y \
    libpulse-dev \
    pkg-config \
    curl \
    unzip \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/* && \
    rustup component add rustfmt

# Vosk SDK (needed at runtime for Vosk engine)
COPY lib/ /build-libs/
RUN if [ "$ENGINE" = "vosk" ] || [ "$ENGINE" = "both" ]; then \
        if [ -f /build-libs/vosk/libvosk.so ]; then \
            echo "Using local libvosk.so from lib/vosk/" && \
            cp /build-libs/vosk/libvosk.so /usr/local/lib/; \
        else \
            echo "Downloading Vosk SDK from GitHub..." && \
            curl -L -o /tmp/vosk.zip \
                https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-x86_64-0.3.45.zip && \
            unzip -q /tmp/vosk.zip -d /tmp/vosk && \
            cp /tmp/vosk/libvosk.so /usr/local/lib/; \
            rm -rf /tmp/vosk.zip /tmp/vosk; \
        fi && \
        ldconfig; \
        mkdir -p /runtime-libs && \
        cp /usr/local/lib/libvosk.so* /runtime-libs/; \
    else \
        mkdir -p /runtime-libs; \
    fi

COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN FEATURES="whisper,tui" && \
    cargo build --release --features "$FEATURES" && \
    cp target/release/audiosub /audiosub

# ============================================================
# Runtime: minimal Debian image
# ============================================================
FROM debian:trixie-slim
ARG ENGINE

RUN apt-get update && apt-get install -y \
    libpulse0 \
    pulseaudio-utils \
    ca-certificates \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Conditionally copy Vosk runtime libs (if any were built)
COPY --from=builder /runtime-libs/ /runtime-libs/
RUN if [ -n "$(ls -A /runtime-libs/ 2>/dev/null)" ]; then \
        cp /runtime-libs/* /usr/local/lib/ && ldconfig; \
    fi

COPY --from=builder /audiosub /usr/local/bin/audiosub

# Default config (can be overridden via env or volume)
ENV RUST_LOG=info

ENTRYPOINT ["audiosub"]
