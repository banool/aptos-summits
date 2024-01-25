# syntax=docker/dockerfile:1.4
FROM rust:1.75-bullseye as builder

WORKDIR /repo
COPY --link . /repo/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --config net.git-fetch-with-cli=true --release -p api

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y build-essential ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --link --from=builder /repo/target/release/api /usr/local/bin/api

# Install Xvfb and other dependencies for GUI applications
RUN apt-get update && apt-get install -y \
    xvfb \
    x11-utils \
    libxcursor1 \
    libxinerama1 \
    libgl1-mesa-glx \
    libxkbcommon-x11-0 \
    && rm -rf /var/lib/apt/lists/*

# Set up the virtual framebuffer X server to run in the background
ENV DISPLAY=:1
CMD Xvfb :1 -screen 0 1024x768x16 &

ENTRYPOINT ["api"]
