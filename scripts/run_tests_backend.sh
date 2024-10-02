#! /bin/bash

# Script to run all the (Rust) tests
# It launches a temporary PostgreSQL container, runs the migrations, runs the tests and then stops and removes the container

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

# Set the DATABASE_URL environment variable
export DATABASE_URL="postgres://postgres:postgres@localhost:${PORT}/nittei"

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
  cargo test --workspace $1 -- --nocapture
else
  # If not in debug mode, run the tests with `cargo-pretty-test`
  cargo-pretty-test --workspace $1
fi

# Store result
RESULT=$?

# Format TS code
pnpm run format

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
