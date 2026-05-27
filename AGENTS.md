# AGENTS.md

This file provides guidance to AI agents when working with code in this repository.

## What is Nittei

Nittei is a self-hosted calendar and scheduler server (Rust + Axum + PostgreSQL). It supports multi-tenancy via Accounts, recurring calendar events, booking/scheduling services, reminders, and optional Google/Outlook calendar integrations.

## Commands

```sh
just setup                        # Start local Postgres container + run migrations
just dev                          # Start dev server with auto-reload (watchexec)
just test                         # Run Rust + JS tests (spins up ephemeral Postgres, then teardown)
just test-backend                 # Run Rust tests only
just test-backend <name>          # Run a specific Rust test by name
just test-frontend                # Run JS tests only (spins up backend automatically)
just test-frontend <file>         # Run a single JS test file
just lint                         # nightly fmt check + clippy
just format                       # Format Rust (nightly) and JS (biome) code
just prepare_sqlx                 # Regenerate SQLx offline query files (run after SQL changes)
just generate-ts-types            # Generate TS types from Rust structs via ts-rs
```

For JS client tests (requires backend running via `just dev`):

```sh
cd clients/javascript && pnpm run test
```

Default local DB: `postgresql://postgres:postgres@localhost:45432/nittei`

## Architecture

The workspace has five Rust crates plus two SDK clients:

| Crate                | Role                                                                                                                                                     |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/domain`      | Pure domain types and business logic — no I/O. All entities (`CalendarEvent`, `Schedule`, `Service`, etc.) and recurrence/booking computation live here. |
| `crates/api_structs` | Shared request/response DTOs between the HTTP API and the Rust SDK client. Decorated with `ts-rs` attributes to auto-generate TypeScript types.          |
| `crates/api`         | Axum HTTP handlers. Each resource module (event, calendar, user, …) has one file per endpoint that implements a `UseCase`.                               |
| `crates/infra`       | PostgreSQL repository implementations (`Repos`). Also contains Google/Outlook OAuth and calendar sync services. SQLx queries live here.                  |
| `crates/utils`       | App configuration (env vars) and shared utilities.                                                                                                       |
| `clients/rust`       | Rust SDK that wraps the HTTP API.                                                                                                                        |
| `clients/javascript` | TypeScript SDK (built with tsdown).                                                                                                                      |

### Request lifecycle

1. An Axum handler in `crates/api` receives a request, extracts `Extension<NitteiContext>` (contains `Repos` + `Config`), and constructs a `UseCase` struct.
2. `execute(usecase, ctx)` or `execute_with_policy(usecase, policy, ctx)` runs the use case — see `crates/api/src/shared/usecase.rs`.
3. After success, `Subscriber` implementations run as side effects (e.g. creating reminders on event creation, syncing to Google/Outlook).
4. The handler converts the use-case result to a JSON response.

### Repository pattern

Every storage type has a trait (e.g. `IEventRepo`) and a `PostgresEventRepo` impl. `Repos` in `crates/infra/src/repos/mod.rs` holds `Arc<dyn IRepo>` for each resource, enabling test mocking. Migrations live in `crates/infra/migrations/`.

### Configuration

All env vars are prefixed `NITTEI__` with `__` as separator (e.g. `NITTEI__PG__DATABASE_URL`, `NITTEI__HTTP_PORT`). See `crates/utils/src/config.rs` for all options. Config is loaded once at startup into the global `APP_CONFIG`.

### SQLx compile-time query checking

SQLx checks SQL queries at compile time using offline snapshots in `crates/infra/.sqlx/`. After adding or changing SQL queries, run `just prepare_sqlx` to regenerate them.

## Lint rules (enforced by Clippy)

- `unwrap()` and `expect()` are **denied** — use `?` or proper error handling.
- `print!` / `println!` / `eprintln!` are **denied** — use `tracing::info!` etc.
- `unsafe` code is **forbidden**.

## TypeScript type generation

Rust domain types annotated with `#[derive(TS)]` auto-generate TypeScript types into `clients/javascript/lib/gen_types/` via `just generate-ts-types`. Do not edit those generated files manually.
