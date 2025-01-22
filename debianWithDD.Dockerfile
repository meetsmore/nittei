# This Dockerfile is based on the debian.Dockerfile and adds the ddprof tool to the image.

# Usage:
# docker buildx build -f debianWithDD.Dockerfile -t image:tag --build-arg='ARCH=x86_64' --platform linux/amd64 .
# docker buildx build -f alpineWithDD.Dockerfile -t image:tag --build-arg='ARCH=aarch64' --platform linux/arm64 .

ARG RUST_VERSION=1.84.0
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
  cp ./target/release/$APP_NAME /bin/server

# Install ddprof
RUN curl -Lo ddprof-linux.tar.xz https://github.com/DataDog/ddprof/releases/latest/download/ddprof-amd64-linux.tar.xz && \
  tar xvf ddprof-linux.tar.xz && \
  mv ddprof/bin/ddprof /ddprof

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

COPY --from=builder /bin/server /bin/
COPY --from=builder /ddprof /ddprof

CMD ["/ddprof", "/bin/server"]