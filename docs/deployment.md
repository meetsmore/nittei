## Deployment of Nittei

Use the docker image: `ghcr/meetsmore/nittei`.

Or build from source:

```bash
cargo run --release
```

Then set up a postgres db with the init script specified [here](../crates/infra/migrations/dbinit.sql).
Lastly provide the following environment variables to the `nittei` server:

```bash
# The connection string to the database
DATABASE_URL
```
