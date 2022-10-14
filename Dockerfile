FROM rust:1-alpine3.16

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev cmake librdkafka-dev ninja build-base
WORKDIR /app
COPY ./ /app
RUN cargo build --release
RUN strip target/release/geyser-neon-filter

FROM alpine:3.16

RUN apk add --no-cache libgcc
COPY --from=0 /app/target/release/geyser-neon-filter .
ENTRYPOINT ["/geyser-neon-filter"]
