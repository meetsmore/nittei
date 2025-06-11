# This Dockerfile is based on the debian.Dockerfile and adds the ddprof tool to the image.

# Usage:
# docker buildx build -f debianWithDD.Dockerfile -t image:tag --build-arg='ARCH=x86_64' --platform linux/amd64 .
# docker buildx build -f debianWithDD.Dockerfile -t image:tag --build-arg='ARCH=aarch64' --platform linux/arm64 .

ARG RUST_VERSION=1.85.1
ARG APP_NAME=nittei
ARG ARCH=x86_64

FROM rust:${RUST_VERSION}-slim AS builder

ARG ARCH=x86_64
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

# Install ddprof
RUN ARCH_IN_URL=$(case "${ARCH}" in \
  x86_64) echo "amd64" ;; \
  aarch64) echo "arm64" ;; \
  *) echo "unsupported-arch" && exit 1 ;; \
  esac) && \
  curl -Lo ddprof-linux.tar.xz https://github.com/DataDog/ddprof/releases/latest/download/ddprof-${ARCH_IN_URL}-linux.tar.xz && \
  tar xvf ddprof-linux.tar.xz && \
  mv ddprof/bin/ddprof /ddprof

# Use the distroless base image for final image
FROM gcr.io/distroless/cc-debian12

# Enable backtraces
ENV RUST_BACKTRACE=1

USER nonroot:nonroot

COPY --from=builder --chown=nonroot:nonroot /nittei /nittei
COPY --from=builder --chown=nonroot:nonroot /nittei-migrate /nittei-migrate
COPY --from=builder --chown=nonroot:nonroot /ddprof /ddprof

CMD ["/ddprof", "--preset", "cpu_live_heap", "/nittei"]