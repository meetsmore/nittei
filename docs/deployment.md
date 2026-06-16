## Deployment of Nittei

Use the docker image: `ghcr/meetsmore/nittei`.

Or build from source:

```bash
cargo run --release
```

### mold

The production Docker builds install [`mold`](https://github.com/rui314/mold) and configure Rust to use it as the linker. This keeps release and profiling builds faster, especially when LTO is enabled.

### Database

Then set up a postgres db with the init script specified [here](../crates/infra/migrations/dbinit.sql).
Lastly provide the following environment variables to the `nittei` server:

```bash
# The connection string to the database
DATABASE_URL
```
