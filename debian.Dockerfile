# Usage:
# docker buildx build -f debian.Dockerfile -t image:tag --build-arg='ARCH=x86_64' --platform linux/amd64 .
# docker buildx build -f debian.Dockerfile -t image:tag --build-arg='ARCH=aarch64' --platform linux/arm64 .

ARG RUST_VERSION=1.85.1
ARG APP_NAME=nittei

FROM rust:${RUST_VERSION}-slim AS builder
ARG APP_NAME

WORKDIR /app/${APP_NAME}

RUN apt update \
  && apt install -y --no-install-recommends curl openssl ca-certificates pkg-config build-essential libssl-dev \
  && apt clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN --mount=type=bind,source=bins,target=/app/${APP_NAME}/bins \
  --mount=type=bind,source=crates,target=/app/${APP_NAME}/crates \
  --mount=type=bind,source=clients,target=/app/${APP_NAME}/clients \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/${APP_NAME}/target/ \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --locked --release && \
  cp ./target/release/$APP_NAME /nittei && \
  cargo build --locked --release --bin nittei-migrate && \
  cp ./target/release/nittei-migrate /nittei-migrate

FROM debian:stable-slim

# Enable backtraces
ENV RUST_BACKTRACE=1

RUN apt update \
  && apt install -y openssl ca-certificates \
  && apt clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

ARG UID=10001
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "${UID}" \
  appuser
USER appuser

COPY --from=builder /nittei /nittei
COPY --from=builder /nittei-migrate /nittei-migrate

CMD ["/nittei"]