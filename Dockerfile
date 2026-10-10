# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.99.0

FROM rust:${RUST_VERSION}-slim-bookworm AS build
WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential cmake perl pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release --bin mqtt-client

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 mqtt \
    && useradd --uid 10001 --gid 10001 --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin mqtt
COPY --from=build --chown=10001:10001 /src/target/release/mqtt-client /usr/local/bin/mqtt-client
COPY --chown=10001:10001 LICENSE /usr/share/licenses/xmqr-client/LICENSE
COPY --chown=10001:10001 docs/third-party-licenses.md /usr/share/licenses/xmqr-client/third-party-licenses.md
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/mqtt-client"]
