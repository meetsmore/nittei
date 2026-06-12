#!/usr/bin/env bash
set -euo pipefail

profile="${BUILD_PROFILE:-release}"
target_dir="${CARGO_TARGET_DIR:-target}"
out_dir="${OUT_DIR:-/tmp/nittei-build}"
os="$(uname -s)"
parallelism="${CARGO_SONIC_PARALLELISM:-2}"

mkdir -p "$out_dir"

if [[ "$os" == "Linux" ]]; then
  arch="${TARGETARCH:-$(uname -m)}"

  # Non-baseline target CPUs to build payloads for. The architecture baseline
  # (x86-64 / generic) is always built implicitly by cargo-sonic; do not list it.
  #
  # Each extra CPU is a full extra payload build, so size and build time scale
  # linearly with the list.
  case "$arch" in
    amd64|x86_64)
      # v3 = Haswell+/Zen+ (AVX2, BMI2). v4 = AVX-512 (Sapphire Rapids, Zen 4+).
      target_cpus="x86-64-v3,x86-64-v4"
      ;;
    arm64|aarch64)
      # n1 = AWS Graviton2 / Ampere Altra. v1 = AWS Graviton3 (SVE).
      target_cpus="neoverse-n1,neoverse-v1"
      ;;
    *)
      echo "Unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  echo "Building binaries"

  # cargo-sonic builds a CPU-dispatched fat binary: a small launcher plus
  # per-target-CPU payloads in an adjacent <bin>.bundle/ directory.
  # NOTE: --release and --profile are mutually exclusive in modern cargo, so
  # only --profile is passed (covers both `release` and `release-dd`).
  # Only the server binary benefits from CPU dispatch; the migration binary is
  # a one-shot tool and is built with plain cargo to avoid artifact collisions
  # in the sonic output directory.
  cargo sonic \
    --target-cpus="$target_cpus" \
    --loader=bundle \
    --parallelism="$parallelism" \
    build \
    --package nittei \
    --bin nittei \
    --locked \
    --profile "$profile"

  echo "Building nittei-migrate binary"

  cargo sonic \
    --target-cpus="$target_cpus" \
    --loader=embedded \
    --parallelism="$parallelism" \
    build \
    --package nittei \
    --bin nittei-migrate \
    --locked \
    --profile "$profile"

  echo "Copying binaries to $out_dir"

  # Final launchers live at exactly: target/sonic/<triple>/<profile>/<bin-name>
  # Bundle dirs live alongside them as: target/sonic/<triple>/<profile>/<bin-name>.bundle/
  # Pin mindepth/maxdepth to avoid picking up intermediate per-payload artifacts.
  find "$target_dir/sonic" -mindepth 3 -maxdepth 3 -type f -name 'nittei' \
    -exec cp {} "$out_dir/" \;

  find "$target_dir/sonic" -mindepth 3 -maxdepth 3 -type d -name '*.bundle' \
    -exec cp -a {} "$out_dir/" \;

  find "$target_dir/sonic" -mindepth 3 -maxdepth 3 -type f -name 'nittei-migrate' \
    -exec cp {} "$out_dir/" \;

else
  # cargo-sonic only supports Linux x86_64/aarch64 (memfd_create + execveat).
  # On other platforms (e.g. macOS for local dev), fall back to a plain build.
  echo "Non-Linux host ($os) detected; falling back to plain 'cargo build' (cargo-sonic is Linux-only)." >&2

  cargo build \
    --package nittei \
    --bin nittei \
    --bin nittei-migrate \
    --locked \
    --profile "$profile"

  cp "$target_dir/$profile/nittei" "$out_dir/nittei"
  cp "$target_dir/$profile/nittei-migrate" "$out_dir/nittei-migrate"
fi
