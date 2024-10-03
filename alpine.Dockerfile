# Usage:
# docker buildx build -f musl.Dockerfile -t image:tag --build-arg='ARCH=x86_64' --platform linux/amd64 .
# docker buildx build -f musl.Dockerfile -t image:tag --build-arg='ARCH=aarch64' --platform linux/arm64 .

ARG ARCH=x86_64
FROM messense/rust-musl-cross:${ARCH}-musl AS builder

ARG ARCH=x86_64
ARG APP_NAME=nittei

# Copy source code from previous stage
COPY . .

# Build application
RUN cargo build --release --target ${ARCH}-unknown-linux-musl && \
  cp ./target/${ARCH}-unknown-linux-musl/release/${APP_NAME} /${APP_NAME}

#Create a new stage with a minimal image
FROM alpine:3.20.3

ARG APP_NAME=nittei
ENV APP_NAME=${APP_NAME}

COPY --from=builder /${APP_NAME} /${APP_NAME}

CMD /${APP_NAME}
