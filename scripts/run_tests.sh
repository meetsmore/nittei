#! /bin/bash

# Script to run all the tests (Rust and JS)
# It
# - launches a temporary PostgreSQL container
# - runs the migrations
# - runs the Rust tests
# - launches the server in background
# - runs the JS tests
# - stops and removes the container
# - stops the server

# Function to clean up the containers
CLEANUP_CALLED=false
cleanup() {
  if [ "$CLEANUP_CALLED" = true ]; then
    return
  fi
  echo "Cleaning up..."
  CLEANUP_CALLED=true

  if [ -n "$NC_PID" ] && kill -0 $NC_PID 2>/dev/null; then
    kill $NC_PID >/dev/null 2>&1
  fi

  if [ "$(docker ps -q -f name=$RANDOM_NAME)" ]; then
    docker stop $RANDOM_NAME >/dev/null 2>&1
  fi

  if [ "$(docker ps -q -f name=ryuk)" ]; then
    docker stop ryuk >/dev/null 2>&1
  fi

  if [ -n "$BACKEND_PID" ]; then
    # Stop the backend server
    kill $BACKEND_PID >/dev/null 2>&1
  fi
}

# Set up a trap to call the cleanup function on EXIT, SIGINT, and SIGTERM
trap cleanup EXIT SIGINT SIGTERM

# Search for a free port to bind the temporary PG container
BASE_PORT=1234
INCREMENT=1

PORT=$BASE_PORT
IS_FREE=$(netstat -taln | grep $PORT)

while [[ -n "$IS_FREE" ]]; do
  PORT=$((PORT + INCREMENT))
  IS_FREE=$(netstat -taln | grep $PORT)
done

# Generate a random name for the temporary PG container
RANDOM_NAME="pg_test_$(date +%s)"

LABEL="nittei_testing=true"

cargo build --workspace

# Launch the resource reaper (like testcontainers)
docker run -d --name ryuk --rm -v /var/run/docker.sock:/var/run/docker.sock -e RYUK_VERBOSE=true -e RYUK_PORT=8080 -p 8080:8080 testcontainers/ryuk:0.8.1 >/dev/null 2>&1

# Keep the connection open and send the label to Ryuk
TIMEOUT=60
(
  echo "label=${LABEL}"
  # Keep the connection open to Ryuk and read the ACK response
  while [ $((TIMEOUT--)) -gt 0 ]; do
    sleep 1
  done
) | nc localhost 8080 &
>/dev/null 2>&1
NC_PID=$!

# Launch a PG container
docker run --rm -d -l ${LABEL} --name $RANDOM_NAME -p $PORT:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=nittei postgres:13 >/dev/null 2>&1

# Set DATABASE_URL (migrations) and NITTEI__DATABASE_URL (app) environment variables
export DATABASE_URL="postgres://postgres:postgres@localhost:${PORT}/nittei"
export NITTEI__DATABASE_URL="$DATABASE_URL"

# Wait for PostgreSQL to be ready
RETRIES=5
until
  docker exec $RANDOM_NAME pg_isready >/dev/null 2>&1 ||
    [ $((RETRIES--)) -eq 0 ]
do
  sleep 1
done

# Run the migrations
cd crates/infra && sqlx migrate run && cd ../..

# Run the tests
# Argument
TEST_NAME=$1

# Add `-- --nocapture` if DEBUG is set
if [ -n "$DEBUG" ]; then
  cargo test --workspace $1 -- --nocapture --skip export_bindings_
else
  # If not in debug mode, run the tests with `cargo-pretty-test`
  cargo-pretty-test --workspace $1 -- --skip export_bindings_
fi

# Store result
RESULT=$?

if [ $RESULT -ne 0 ]; then
  echo ""
  echo "#################"
  echo "Some tests failed!"
  echo "#################"
  echo ""
  exit $RESULT
fi

#### FRONTEND ####

# Search for a free port to bind the temporary backend server
BASE_PORT=1234
INCREMENT=1

PORT=$BASE_PORT
IS_FREE=$(netstat -taln | grep $PORT)

while [[ -n "$IS_FREE" ]]; do
  PORT=$((PORT + INCREMENT))
  IS_FREE=$(netstat -taln | grep $PORT)
done

export NITTEI_PORT=$PORT

# Launch the backend server
if [ -n "$DEBUG" ]; then
  # If in debug, log the output
  cargo run --bin nittei -- --port $PORT &
else
  # If not in debug, run in the background and silently
  cargo run --bin nittei -- --port $PORT >/dev/null 2>&1 &
fi

# Save the PID of the backend server
BACKEND_PID=$!

# Wait for backend server to be ready
RETRIES=5
until
  curl localhost:$PORT/api/v1/healthcheck >/dev/null 2>&1 ||
    [ $((RETRIES--)) -eq 0 ]
do
  sleep 1
done

# Run JS tests
pnpm run test

# Store result
RESULT=$?

if [ $RESULT -ne 0 ]; then
  echo ""
  echo "#################"
  echo "Some tests failed!"
  echo "#################"
  echo ""
  exit $RESULT
fi

echo ""
echo "#################"
echo "All tests passed!"
echo "#################"
echo ""

# The cleanup function will be called automatically
