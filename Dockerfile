FROM rust:1.89-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/lastfm-rust /usr/local/bin/
CMD ["lastfm-rust"]