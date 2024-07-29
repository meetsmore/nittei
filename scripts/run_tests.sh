#! /bin/bash

# Script to run all the (Rust) tests of the scheduler project
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

cd scheduler && cargo build --workspace

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
docker run --rm -d -l ${LABEL} --name $RANDOM_NAME -p $PORT:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=scheduler postgres:13 >/dev/null 2>&1

# Set the DATABASE_URL environment variable
export DATABASE_URL="postgres://postgres:postgres@localhost:${PORT}/scheduler"

# Run the migrations
cd crates/infra && sqlx migrate run && cd ../..

# Run the tests
cargo nextest run --workspace && cd ..

# The cleanup function will be called automatically