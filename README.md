# Nittei

[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://github.com/meetsmore/nittei/actions/workflows/release.yml/badge.svg)](https://github.com/meetsmore/nittei/actions/workflows/release.yml)

## Overview

`Nittei` is a self-hosted calendar and scheduler server built upon [nittei](https://github.com/fmeringdal/nettu-scheduler).

It supports authentication through

- API keys for server - server
- JSON Web Tokens for browser - server

## Concepts

- **Accounts**: `Account`s are the way to handle multi-tenancy. Each account can have multiple users
- **Users**: `User`s represent an actual user, and each one can belong to only one account
- **Calendars**: Users can each have multiple `Calendar`s, which are used for grouping `Calendar Event`s
- **Calendar Events**: `Calendar Event`s are the actual events. They support recurrence rules and be flexible queried
- **Reminders**: `Calendar Event`s can have reminders
- **Freebusy**: `User`s can be queried on their free busy
- **Metadata**: `Calendar`s and `Calendar Event`s can contain metadata values, allowing to store anything alongside them

More advanced features include

- **Booking**: Create a `Service` and register `User`s on it to make them bookable
- **Integrations**: Connect your Nittei, Google and Outlook calendars
- **Webhooks**: Notifying your server about `Calendar Event` reminders

## Getting started

### Prerequisites

As this application is written in **Rust**, you first need to have it installed. The easiest is to use [Rustup](https://rustup.rs/) and the instructions can be found on the [official website](https://rustup.rs/). The command is the following:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This project uses [Just](https://github.com/casey/just) as a task manager. To install it, you can use Homebrew (MacOS & Linux)

```sh
brew install just
```

Once Rust and Just are installed, a few more utility tools can be installed by running

```sh
just install_tools
```

This will compile and install

- `sqlx-cli`: CLI used for applying the SQL migrations & for generating the offline files for [SQLx](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- `cargo-pretty-test`: CLI for running the tests and print them in a prettier way compared to `cargo test`
- `cargo-watch`: CLI for auto-reloading the backend when source files have changed

### Initial setup

You can launch the Postgres container required for local development and execute the migration by running

```sh
just setup
```

### Launching the server

Now we are ready to start the `Nittei` server

```bash
just dev
```

There are a few environment variables that can be used to control the server, see `crates/utils/src/config.rs` for more details.

Quick example of how to create and query a user

```bash
export SECRET_API_KEY="REPLACE ME WITH YOUR API KEY"

# Create a user with metadata
curl -X POST -H "Content-Type: application/json" -H "x-api-key: $SECRET_API_KEY" -d '{"metadata": { "groupId": "123" }}' http://localhost:5000/api/v1/user

# Get users by metadata
curl -H "x-api-key: $SECRET_API_KEY" "http://localhost:5000/api/v1/user/meta?key=groupId&value=123"
```

Please see below for links to more examples.

### Running the tests

For running all the tests at once, you can run:

```sh
just test-all
```

> This launches an ephemeral PostgreSQL container used by the tests. The script tries to remove the container at the end.

For running only the tests for the server (Rust) and the Rust client SDK, you can simply run:

```sh
just test
```

> This also launches an ephemeral PostgreSQL container.

For running the tests for the JS client SDK, you first need to have the server running. As a reminder, this is the command:

```sh
just dev
```

Once it's running, the tests can be run by doing the following:

```sh
cd clients/javascript/
pnpm run test
```

### OpenAPI UI

The API also provides an OpenAPI UI (Swagger) that allows to see, and test, all endpoints. When running the API in your local environment, you can find it at <http://localhost:5000/swagger-ui/#/>.

### Additional tools

You can install additional tools by running

```sh
just install_all_tools
```

This will install

- `cargo-outdated`: CLI used to list the dependencies that are outdated
- `cargo-udeps`: CLI used to list the dependencies that are unused

## Examples

- [Calendars and Events](examples/calendar-events.md)

- [Booking](examples/booking.md)

- [Reminders](examples/reminders.md)

- [Creating JWT for end-users](examples/jwt.md)

## Main Rust dependencies used

- [Tokio](https://github.com/tokio-rs/tokio): async runtime needed for using async/await
- [Axum](https://github.com/tokio-rs/axum): HTTP framework
- [validator](https://github.com/Keats/validator) + [axum-valid](https://github.com/gengteng/axum-valid): add validation step when deserializing structs and enums
- [SQLx](https://github.com/launchbadge/sqlx): async, pure Rust SQL crate featuring compile-time checked queries
- [serde](https://github.com/serde-rs/serde): serialization (and deserialization) framework
- [anyhow](https://github.com/dtolnay/anyhow): easy to use Error type for applications
- [chrono](https://github.com/chronotope/chrono): library providing structs/enums for handling datetimes and timezones
- [rrule](https://github.com/fmeringdal/rust-rrule): library for generating instances of recurring events
- [ts-rs](https://github.com/Aleph-Alpha/ts-rs): automatically generate Typescript types from Rust structs and enums
- [tracing](https://github.com/tokio-rs/tracing): handles tracing and logging
- [tikv-jemallocator](https://github.com/tikv/jemallocator): replaces default memory allocator with Jemalloc

## New to Rust

You can check [docs/rust.md](./docs/rust.md) to get a small introduction to Rust.

## Contributing

Contributions are welcome and are greatly appreciated!

## License

[MIT](LICENSE)

## Special thanks

- [fmeringdal](https://github.com/fmeringdal/nettu-scheduler) for the initial project. This repository is a fork adapted to our needs.
