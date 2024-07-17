#! /bin/bash

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

# Launch a PG container
docker run -d --name $RANDOM_NAME -p $PORT:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=scheduler postgres:13

# Set the DATABASE_URL environment variable
export DATABASE_URL="postgres://postgres:postgres@localhost:${PORT}/scheduler"

# Run the migrations
cd scheduler/crates/infra && sqlx migrate run && cd ../../..

# Run the tests
cd scheduler && cargo nextest run --workspace && cd ..

# Stop and remove the temporary PG container
docker rm -f $RANDOM_NAME
