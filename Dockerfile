FROM rust:latest AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libpulse-dev \
    pkg-config \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

RUN curl -L -o /tmp/vosk.zip \
    https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-x86_64-0.3.45.zip && \
    unzip /tmp/vosk.zip -d /tmp/vosk && \
    find /tmp/vosk -name 'libvosk.so' -exec cp {} /usr/local/lib/ \; && \
    ldconfig && \
    rm -rf /tmp/vosk.zip /tmp/vosk

COPY Cargo.toml Cargo.lock ./
COPY src src/

RUN LIBRARY_PATH=/usr/local/lib cargo build --release --no-default-features --features "vosk,tui" && \
    cp target/release/audiosub /audiosub

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libpulse0 \
    ca-certificates \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/lib/libvosk.so /usr/local/lib/
COPY --from=builder /audiosub /usr/local/bin/audiosub

RUN mkdir -p /cache/audiosub/models && \
    curl -L -o /tmp/model.zip \
    https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip && \
    unzip /tmp/model.zip -d /cache/audiosub/models/ && \
    rm /tmp/model.zip

RUN ldconfig

ENV AUDIOSUB_MODEL_PATH=/cache/audiosub/models/vosk-model-small-ru-0.22
ENV TZ=UTC

ENTRYPOINT ["audiosub"]
