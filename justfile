export DATABASE_URL := "postgresql://postgres:postgres@localhost:45432/nittei"
export RUST_BACKTRACE := "1"

# Install minimal tools
install_tools: 
	cargo install sqlx-cli
	cargo install cargo-pretty-test
	cargo install --locked watchexec-cli

# Install all tools
install_all_tools: install_tools
	cargo install cargo-outdated
	cargo install cargo-udeps


# Setup
setup: _setup_db _setup_client_node

# Setup database + execute migrations
_setup_db:
	docker compose -f integrations/docker-compose.yml up -d
	cd crates/infra && sqlx migrate run

# Setup Javascript client - run `pnpm install`
_setup_client_node:
	cd clients/javascript && pnpm install

# Dev
dev: _setup_db
	watchexec -r -e rs -- cargo run

# Prepare offline SQLx
prepare_sqlx:
	cd crates/infra && cargo sqlx prepare

# Generate TS types
generate-ts-types:
	bash ./scripts/generate_ts_types.sh

# Run the tests on a temporary DB container
test test_name="":
	bash ./scripts/run_tests_backend.sh {{test_name}}

# Run only the JS tests - this assumes that the backend is running
test-js:
	cd clients/javascript && pnpm run test

# Run the load tests on a temporary DB container
loadtest:
	INCLUDE_LOAD_TESTS=true bash ./scripts/run_tests_backend.sh loadtests

# Run all tests (backend + frontend)
test-all:
	bash ./scripts/run_tests.sh

# Lint
lint: _setup_db
	cargo +nightly fmt --all -- --check
	cargo clippy --verbose

# Format
format:
	cargo +nightly fmt --all
	pnpm run format

# Check unused dependencies
check-unused: _setup_db
	cargo udeps --all-targets

# Check for outdated dependencies
check-update: _setup_db
	cargo outdated -wR
	cargo update --dry-run