# syntax=docker/dockerfile:1.4
FROM rust:1.61.0 AS builder

COPY . /app

WORKDIR /app
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target/release/build \
    --mount=type=cache,target=/app/target/release/deps \
    --mount=type=cache,target=/app/target/release/.fingerprint \
    cargo build --release


FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/ddns-cloudflare /

ENTRYPOINT ["/ddns-cloudflare"]
