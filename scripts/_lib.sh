#! /bin/bash

# Shared library for test scripts.
# Source this file — do not execute it directly.

# ============================================================
# Utility: find a free TCP port starting from a given base
# Usage: PORT=$(find_free_port 1234)
# ============================================================
find_free_port() {
  local PORT=${1:-1234}
  while [[ -n "$(netstat -taln | grep $PORT)" ]]; do
    PORT=$((PORT + 1))
    sleep 0.1
  done
  echo "$PORT"
}

# ============================================================
# Cleanup: stop containers and the backend server (if any)
# Guarded against double-invocation (trap + explicit call).
# ============================================================
CLEANUP_CALLED=false
cleanup() {
  if [ "$CLEANUP_CALLED" = true ]; then
    return
  fi
  echo "Cleaning up..."
  CLEANUP_CALLED=true

  if [ -n "$NC_PID" ] && kill -0 "$NC_PID" 2>/dev/null; then
    kill "$NC_PID" >/dev/null 2>&1
  fi

  if [ "$(docker ps -q -f name="$RANDOM_NAME")" ]; then
    docker stop "$RANDOM_NAME" >/dev/null 2>&1
  fi

  if [ "$(docker ps -q -f name=ryuk)" ]; then
    docker stop ryuk >/dev/null 2>&1
  fi

  if [ -n "$BACKEND_PID" ] && kill -0 "$BACKEND_PID" 2>/dev/null; then
    kill "$BACKEND_PID" >/dev/null 2>&1
  fi
}

# ============================================================
# Setup infrastructure: build, ryuk, postgres, migrations.
# Sets DATABASE_URL and NITTEI__PG__DATABASE_URL in the env.
# ============================================================
setup_infrastructure() {
  PG_PORT=$(find_free_port 1234)
  RANDOM_NAME="pg_test_$(date +%s)"
  LABEL="nittei_testing=true"

  echo ""
  echo "###############"
  echo "Building app..."
  echo "###############"
  echo ""

  cargo build --workspace

  # Launch the resource reaper (like testcontainers)
  docker run -d --name ryuk --rm \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -e RYUK_VERBOSE=true -e RYUK_PORT=8080 \
    -p 8080:8080 \
    testcontainers/ryuk:0.8.1 >/dev/null 2>&1

  # Keep the connection alive and register the container label with Ryuk
  TIMEOUT=60
  (
    echo "label=${LABEL}"
    while [ $((TIMEOUT--)) -gt 0 ]; do
      sleep 1
    done
  ) | nc localhost 8080 >/dev/null 2>&1 &
  NC_PID=$!

  echo ""
  echo "#######################"
  echo "Launching containers..."
  echo "#######################"
  echo ""

  docker run --rm -d -l "${LABEL}" --name "$RANDOM_NAME" \
    -p "$PG_PORT":5432 \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB=nittei \
    postgres:15.4-alpine >/dev/null 2>&1

  export DATABASE_URL="postgres://postgres:postgres@localhost:${PG_PORT}/nittei"
  export NITTEI__PG__DATABASE_URL="$DATABASE_URL"

  # Wait for PostgreSQL to be ready
  RETRIES=5
  until docker exec "$RANDOM_NAME" pg_isready >/dev/null 2>&1 || [ $((RETRIES--)) -eq 0 ]; do
    sleep 1
  done

  # Give it a moment to fully initialize
  sleep 1

  # Run migrations (subshell to avoid changing the caller's working directory)
  (cd crates/infra && sqlx migrate run)
}

# ============================================================
# Run the backend (Rust) tests.
# Pass any extra cargo-test filter arguments via "$@".
# ============================================================
run_backend_tests() {
  echo ""
  echo "########################"
  echo "Running backend tests..."
  echo "########################"
  echo ""

  if [ -n "$DEBUG" ]; then
    cargo test --workspace "$@" -- --nocapture --skip export_bindings_
  else
    cargo-pretty-test --workspace "$@" -- --skip export_bindings_
  fi
}

# ============================================================
# Start the backend server and wait for it to be ready.
# Sets BACKEND_PID and NITTEI_PORT in the env.
# ============================================================
start_backend_server() {
  BACKEND_PORT=$(find_free_port 1234)
  export NITTEI_PORT=$BACKEND_PORT

  echo ""
  echo "#############################################"
  echo "Starting backend server for frontend tests..."
  echo "#############################################"
  echo ""

  if [ -n "$DEBUG" ]; then
    cargo run --bin nittei -- --port "$BACKEND_PORT" &
  else
    cargo run --bin nittei -- --port "$BACKEND_PORT" >/dev/null 2>&1 &
  fi

  BACKEND_PID=$!

  # Wait for the server to become healthy
  RETRIES=10
  until curl -s "localhost:${BACKEND_PORT}/api/v1/health/live" >/dev/null 2>&1 || [ $((RETRIES--)) -eq 0 ]; do
    sleep 1
  done
}

# ============================================================
# Run the frontend (JS) tests.
# Expects the backend server to already be running.
# Optionally pass a test file path to run only that file.
# Usage: run_frontend_tests [<test-file>]
# ============================================================
run_frontend_tests() {
  echo ""
  echo "#########################"
  echo "Running frontend tests..."
  echo "#########################"
  echo ""

  pnpm run test "$@"
}
