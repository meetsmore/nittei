<!-- <div align="center">
<img width="400" src="docs/logo.png" alt="logo">
</div> -->

# Nittei

[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://github.com/meetsmore/nittei/actions/workflows/release.yml/badge.svg)](https://github.com/meetsmore/nittei/actions/workflows/release.yml)

## Overview

`Nittei` is a self-hosted calendar and scheduler server built upon [nittei](https://github.com/fmeringdal/nettu-scheduler).

<!-- It aims to provide the building blocks for building calendar / booking apps with ease. It has a simple REST API and also a [JavaScript SDK](https://www.npmjs.com/package/@nittei/sdk-scheduler) and [Rust SDK](https://crates.io/crates/nittei_sdk). -->

It supports authentication through

- API keys for server - server
- JSON Web Tokens for browser - server

## Features

- **Multi-tenancy**: All resources are grouped by `Account`s
- **Calendars**: For grouping `Calendar Event`s
- **Calendar Events**: Supports recurrence rules, flexible querying and reminders
- **Freebusy**: Find out when `User`s are free and when they are busy
- **Metadata queries**: Add key-value metadata to your resources and then query on that metadata

More advanced features include

- **Booking**: Create a `Service` and register `User`s on it to make them bookable
- **Integrations**: Connect your Nittei, Google and Outlook calendars
- **Webhooks**: Notifying your server about `Calendar Event` reminders

## Quick start

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

There are a few environment variables that can be used to control the server

- `NITTEI__HTTP_HOST` allows to specify the host to be used by the server (e.g. 127.0.0.1 or 0.0.0.0)
- `NITTEI__HTTP_PORT` allows to specify the port to be used by the server
- `NITTEI__DATABASE_URL` allows to specify the database the server should use
- `NITTEI__ACCOUNT__SECRET_KEY` is going to create an `Account` (if it does not already exist) during
  server startup with the given key. `Account`s act as tenants in the server, and it is possible to create multiple `Account`s by using the `NITTEI__CREATE_ACCOUNT_SECRET_CODE` which you can provide as an environment variable.

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

## Contributing

Contributions are welcome and are greatly appreciated!

## License

[MIT](LICENSE)

## Special thanks

- [fmeringdal](https://github.com/fmeringdal/nettu-scheduler) for the initial project. This repository is a fork adapted to our needs.
