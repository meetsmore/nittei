export DATABASE_URL := "postgresql://postgres:postgres@localhost:45432/nettuscheduler"
export RUST_BACKTRACE := "1"

# Install minimal tools
install_tools: 
	cargo install sqlx-cli --no-default-features --features postgres
	cargo install cargo-nextest

# Install all tools
install_all_tools: install_tools
	cargo install cargo-outdated
	cargo install cargo-udeps


# Setup
setup: _setup_db _setup_client_node

# Setup database + execute migrations
_setup_db:
	docker compose -f scheduler/integrations/docker-compose.yml up -d
	cd scheduler/crates/infra && sqlx migrate run

# Setup Javascript client - run `pnpm install`
_setup_client_node:
	cd scheduler/clients/javascript && pnpm install

# Dev
dev: _setup_db
	cd scheduler && cargo run

# Run the tests on a temporary DB container
test:
	bash ./scripts/run_tests.sh

# Lint
lint: _setup_db
	cd scheduler && cargo fmt
	cd scheduler && cargo clippy --verbose

# Check unused dependencies
check-unused: _setup_db
	cd scheduler && cargo udeps --all-targets

# Check for outdated dependencies
check-update: _setup_db
	cd scheduler && cargo outdated -wR
	cd scheduler && cargo update --dry-run