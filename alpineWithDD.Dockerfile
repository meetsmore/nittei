# This Dockerfile is based on the alpine.Dockerfile and adds the ddprof tool to the image.
#
# Usage:
# docker buildx build -f alpineWithDD.Dockerfile -t image:tag --build-arg='ARCH=x86_64' --platform linux/amd64 .
# docker buildx build -f alpineWithDD.Dockerfile -t image:tag --build-arg='ARCH=aarch64' --platform linux/arm64 .

ARG ARCH=x86_64
FROM messense/rust-musl-cross:${ARCH}-musl AS builder

ARG ARCH=x86_64
ARG APP_NAME=nittei
ARG RUST_VERSION=1.85.0

# Install and set the specific Rust version
RUN rustup install ${RUST_VERSION} && rustup default ${RUST_VERSION}

# Install the musl target for the correct architecture
RUN rustup target add ${ARCH}-unknown-linux-musl

# Verify the Rust and target setup
RUN rustc --version && rustup show

# Copy source code from previous stage
COPY . .

# Build application
RUN cargo build --release --target ${ARCH}-unknown-linux-musl && \
  cp ./target/${ARCH}-unknown-linux-musl/release/${APP_NAME} /${APP_NAME}

# Install ddprof
RUN ARCH_IN_URL=$(case "${ARCH}" in \
  x86_64) echo "amd64" ;; \
  aarch64) echo "arm64" ;; \
  *) echo "unsupported-arch" && exit 1 ;; \
  esac) && \
  curl -Lo ddprof-linux.tar.xz https://github.com/DataDog/ddprof/releases/latest/download/ddprof-${ARCH_IN_URL}-linux.tar.xz && \
  tar xvf ddprof-linux.tar.xz && \
  mv ddprof/bin/ddprof /ddprof

#Create a new stage with a minimal image
FROM alpine:3.20.3

ARG APP_NAME=nittei
ENV APP_NAME=${APP_NAME}

COPY --from=builder /${APP_NAME} /${APP_NAME}
COPY --from=builder /ddprof /ddprof

CMD ["/bin/sh", "-c", "exec /ddprof --preset cpu_live_heap /${APP_NAME}"]
