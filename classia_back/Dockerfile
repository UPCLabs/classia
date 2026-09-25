FROM rust:1.95-bookworm AS chef

RUN cargo install cargo-chef --version 0.1.78 --locked

WORKDIR /app

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && groupadd --system --gid 10001 classia \
    && useradd --system --uid 10001 --gid classia --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin classia \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder --chown=10001:10001 /app/target/release/classia_back /usr/local/bin/classia-back

ENV PORT=5000

USER 10001:10001

EXPOSE 5000

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail --silent --show-error "http://127.0.0.1:${PORT}/api/ping" > /dev/null || exit 1

CMD ["classia-back"]
