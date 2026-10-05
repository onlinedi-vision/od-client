FROM rust:bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libglib2.0-dev \
        libgtk-3-dev \
        libpango1.0-dev \
        libwebkit2gtk-4.1-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy

WORKDIR /workspace/src-tauri
