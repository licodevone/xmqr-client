# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.99.0

FROM rust:${RUST_VERSION}-slim-bookworm AS build
WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential cmake perl pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY vendor ./vendor
COPY scripts/collect_notices.py ./scripts/collect_notices.py
COPY packaging/notices ./packaging/notices
RUN apt-get update \
    && apt-get install -y --no-install-recommends python3 \
    && rm -rf /var/lib/apt/lists/* \
    && python3 scripts/collect_notices.py --output /third-party-notices
ENV CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUSTFLAGS=-Cdebuginfo=0
RUN cargo build --locked --release --bin mqtt-client --bin mqtt-bridge

FROM debian:bookworm-slim AS runtime
ARG VERSION=0.42.0
ARG VCS_REF=unknown
LABEL org.opencontainers.image.title="XMQR Client" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.source="https://github.com/licodevone/xmqr-client" \
      org.opencontainers.image.licenses="MIT"
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 mqtt \
    && useradd --uid 10001 --gid 10001 --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin mqtt
COPY --from=build --chown=10001:10001 /src/target/release/mqtt-client /usr/local/bin/mqtt-client
COPY --from=build --chown=10001:10001 /src/target/release/mqtt-bridge /usr/local/bin/mqtt-bridge
COPY --chown=10001:10001 LICENSE /usr/share/licenses/xmqr-client/LICENSE
COPY --chown=10001:10001 docs/third-party-licenses.md /usr/share/licenses/xmqr-client/third-party-licenses.md
COPY --from=build --chown=10001:10001 /third-party-notices /usr/share/licenses/xmqr-client/third-party
USER 10001:10001
STOPSIGNAL SIGTERM
RUN test "$(/usr/local/bin/mqtt-client --version)" = "mqtt-client ${VERSION}"
RUN test "$(/usr/local/bin/mqtt-bridge --version)" = "mqtt-bridge 0.3.0 (xmqr-client ${VERSION})"
ENTRYPOINT ["/usr/local/bin/mqtt-client"]
