#! /bin/bash

# Run the frontend (JS/TS) tests only.
# - Builds the app
# - Launches a temporary PostgreSQL container
# - Runs migrations
# - Starts the backend server (required by the JS tests)
# - Runs the JS tests
# - Cleans up on exit
#
# Usage: ./run_tests_frontend.sh [<test-file>]
#        DEBUG=1 ./run_tests_frontend.sh   (verbose backend output)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/_lib.sh"

trap cleanup EXIT SIGINT SIGTERM

setup_infrastructure
start_backend_server
run_frontend_tests "$@"
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
