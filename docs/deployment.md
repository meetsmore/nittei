## Deployment of Nittei

Use the docker image: `ghcr/meetsmore/nittei`.

Or build from source:

```bash
cargo run --release
```

### mold

The production Docker builds install [`mold`](https://github.com/rui314/mold) and configure Rust to use it as the linker. This keeps release and profiling builds faster, especially when LTO is enabled.

### cargo-sonic

The Debian Docker images use [`cargo-sonic`](https://docs.rs/crate/cargo-sonic/latest) to build CPU-dispatched binaries. The image contains a baseline payload plus an optimized payload for the target architecture: `x86-64-v3` on `amd64`, and `neoverse-n1` on `arm64`.

The build uses `--loader=bundle`, so the final image must include both the launcher binaries and their adjacent `.bundle` directories.

### Database

Then set up a postgres db with the init script specified [here](../crates/infra/migrations/dbinit.sql).
Lastly provide the following environment variables to the `nittei` server:

```bash
# The connection string to the database
DATABASE_URL
```
