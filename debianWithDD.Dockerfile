# This Dockerfile is based on the debian.Dockerfile and adds the ddprof tool to the image.

# Usage:
# docker buildx build -f debianWithDD.Dockerfile -t image:tag --platform linux/amd64 .
# docker buildx build -f debianWithDD.Dockerfile -t image:tag --platform linux/arm64 .

FROM rust:1.96.0-slim-trixie AS builder

WORKDIR /app/nittei

ARG TARGETARCH
ENV BUILD_PROFILE=release-dd
# RUSTFLAGS overrides .cargo/config.toml rustflags, so mold must be set explicitly here.
ENV RUSTFLAGS="-C force-frame-pointers=yes -C link-arg=-fuse-ld=mold"

RUN apt update \
  && apt install -y --no-install-recommends curl openssl ca-certificates pkg-config build-essential libssl-dev mold \
  && apt clean \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

COPY .cargo .cargo
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./clients/rust ./clients/rust
COPY ./crates ./crates
COPY ./bins ./bins

RUN mkdir -p /tmp/nittei-build \
  && cargo build --locked --profile ${BUILD_PROFILE} --bin nittei --bin nittei-migrate \
  && cp ./target/${BUILD_PROFILE}/nittei /tmp/nittei-build/nittei \
  && cp ./target/${BUILD_PROFILE}/nittei-migrate /tmp/nittei-build/nittei-migrate

# Install ddprof
RUN ARCH_IN_URL=$(case "${TARGETARCH:-$(uname -m)}" in \
  amd64|x86_64) echo "amd64" ;; \
  arm64|aarch64) echo "arm64" ;; \
  *) echo "unsupported-arch" && exit 1 ;; \
  esac) && \
  curl -Lo ddprof-linux.tar.xz https://github.com/DataDog/ddprof/releases/latest/download/ddprof-${ARCH_IN_URL}-linux.tar.xz && \
  tar xvf ddprof-linux.tar.xz && \
  mv ddprof/bin/ddprof /ddprof

# Use the distroless base image for final image
FROM gcr.io/distroless/cc-debian13

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

COPY --from=builder --chown=nonroot:nonroot /tmp/nittei-build/ /
COPY --from=builder --chown=nonroot:nonroot /ddprof /ddprof

CMD ["/ddprof", "--preset", "cpu_live_heap", "/nittei"]
