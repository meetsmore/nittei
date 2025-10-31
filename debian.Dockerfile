# Usage:
# docker buildx build -f debian.Dockerfile -t image:tag --build-arg='ARCH=x86_64' --platform linux/amd64 .
# docker buildx build -f debian.Dockerfile -t image:tag --build-arg='ARCH=aarch64' --platform linux/arm64 .

FROM rust:1.90.1-slim AS builder

WORKDIR /app/nittei

RUN apt update \
  && apt install -y --no-install-recommends curl openssl ca-certificates pkg-config build-essential libssl-dev \
  && apt clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

COPY ./Cargo.toml ./Cargo.lock .cargo ./
COPY ./clients/rust ./clients/rust
COPY ./crates ./crates
COPY ./bins ./bins

RUN cargo build --locked --release && \
  cp ./target/release/nittei /nittei && \
  cargo build --locked --release --bin nittei-migrate && \
  cp ./target/release/nittei-migrate /nittei-migrate

# Use the distroless base image for final image
FROM gcr.io/distroless/cc-debian12

# Set the git repository url and commit hash for DD
ARG GIT_REPO_URL
ARG GIT_COMMIT_HASH

ENV DD_GIT_REPOSITORY_URL=${GIT_REPO_URL}
ENV DD_GIT_COMMIT_SHA=${GIT_COMMIT_HASH}
ENV DD_SOURCE_CODE_PATH_MAPPING="/app/nittei/bins:/bins,/app/nittei/crates:/crates"

# Set the backtrace level by default to 1
ARG RUST_BACKTRACE=1
ENV RUST_BACKTRACE=${RUST_BACKTRACE}

USER nonroot:nonroot

COPY --from=builder --chown=nonroot:nonroot /nittei /nittei
COPY --from=builder --chown=nonroot:nonroot /nittei-migrate /nittei-migrate

CMD ["/nittei"]