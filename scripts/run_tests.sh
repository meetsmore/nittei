#! /bin/bash

# Run all tests (Rust backend + JS frontend).
# - Builds the app once
# - Launches a temporary PostgreSQL container (shared by both test suites)
# - Runs migrations
# - Runs Rust tests
# - Starts the backend server
# - Runs JS tests
# - Cleans up on exit
#
# Usage: ./run_tests.sh [<backend-test-name-filter>]
#        DEBUG=1 ./run_tests.sh   (verbose output)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/_lib.sh"

trap cleanup EXIT SIGINT SIGTERM

setup_infrastructure

# ---- Backend tests ----
run_backend_tests "$@"
RESULT=$?

if [ $RESULT -ne 0 ]; then
  echo ""
  echo "#################"
  echo "Some tests failed!"
  echo "#################"
  echo ""
  exit $RESULT
fi

# ---- Frontend tests ----
start_backend_server
run_frontend_tests
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
