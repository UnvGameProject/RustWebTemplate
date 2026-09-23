FROM rust:1.98.1-bookworm AS builder

WORKDIR /build

COPY tools/rusty-cli/Cargo.toml \
     tools/rusty-cli/Cargo.lock \
     ./

COPY tools/rusty-cli/src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder \
    /build/target/release/rusty \
    /usr/local/bin/rusty

WORKDIR /workspace

CMD ["sleep", "infinity"]