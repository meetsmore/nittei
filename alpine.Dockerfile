# Usage:
# docker buildx build -f alpine.Dockerfile -t image:tag --platform linux/amd64 .
# docker buildx build -f alpine.Dockerfile -t image:tag --platform linux/arm64 .

ARG TARGETARCH=amd64

FROM messense/rust-musl-cross:x86_64-musl AS builder-amd64
ENV MUSL_TARGET=x86_64-unknown-linux-musl

FROM messense/rust-musl-cross:aarch64-musl AS builder-arm64
ENV MUSL_TARGET=aarch64-unknown-linux-musl

FROM builder-${TARGETARCH} AS builder

ARG RUST_VERSION=1.95.0
ENV BUILD_PROFILE=release

# Install and set the specific Rust version
RUN rustup install ${RUST_VERSION} && rustup default ${RUST_VERSION}

# Install the musl target for the correct architecture
RUN rustup target add ${MUSL_TARGET}

# Verify the Rust and target setup
RUN rustc --version && rustup show

# Copy source code
COPY .cargo .cargo
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./crates ./crates
COPY ./bins ./bins
COPY ./clients/rust ./clients/rust

RUN cargo build --locked --profile ${BUILD_PROFILE} --target ${MUSL_TARGET} \
  --bin nittei \
  --bin nittei-migrate

RUN mkdir -p /out \
  && cp ./target/${MUSL_TARGET}/${BUILD_PROFILE}/nittei /out/nittei \
  && cp ./target/${MUSL_TARGET}/${BUILD_PROFILE}/nittei-migrate /out/nittei-migrate

# Create a new stage with a minimal image
FROM alpine:3.23.4

# Set the git repository url and commit hash for DD
ARG GIT_REPO_URL
ARG GIT_COMMIT_HASH

ENV DD_GIT_REPOSITORY_URL=${GIT_REPO_URL}
ENV DD_GIT_COMMIT_SHA=${GIT_COMMIT_HASH}
ENV DD_SOURCE_CODE_PATH_MAPPING="/app/nittei/bins:/bins,/app/nittei/crates:/crates"

# Set the backtrace level by default to 1
ARG RUST_BACKTRACE=1
ENV RUST_BACKTRACE=${RUST_BACKTRACE}

COPY --from=builder /out/ /

CMD ["/nittei"]
