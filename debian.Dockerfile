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

# Use the distroless base image for final image
FROM gcr.io/distroless/cc-debian12

# Set the git repository url and commit hash for DD
ARG GIT_REPO_URL
ARG GIT_COMMIT_HASH

ENV DD_GIT_REPOSITORY_URL=${GIT_REPO_URL}
ENV DD_GIT_COMMIT_SHA=${GIT_COMMIT_HASH}

# Set the backtrace level by default to 1
ARG RUST_BACKTRACE=1
ENV RUST_BACKTRACE=${RUST_BACKTRACE}

USER nonroot:nonroot

COPY --from=builder --chown=nonroot:nonroot /nittei /nittei
COPY --from=builder --chown=nonroot:nonroot /nittei-migrate /nittei-migrate

CMD ["/nittei"]