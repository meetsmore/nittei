#!/usr/bin/env bash
set -euo pipefail

profile="${BUILD_PROFILE:-release}"
arch="${TARGETARCH:-$(uname -m)}"
target_dir="${CARGO_TARGET_DIR:-target}"
out_dir="${OUT_DIR:-/out}"

case "$arch" in
  amd64|x86_64)
    target_cpus="x86-64-v3"
    ;;
  arm64|aarch64)
    target_cpus="neoverse-n1"
    ;;
  *)
    echo "Unsupported architecture: $arch" >&2
    exit 1
    ;;
esac

cargo sonic \
  --target-cpus="$target_cpus" \
  --loader=bundle \
  build --locked \
  --profile "$profile" \
  --bin nittei \
  --bin nittei-migrate

mkdir -p "$out_dir"
find "$target_dir/sonic" -type f \( \
  -name 'nittei' -o \
  -name 'nittei-migrate' \
\) -exec cp {} "$out_dir/" \;

find "$target_dir/sonic" -type d -name '*.bundle' -exec cp -a {} "$out_dir/" \;
