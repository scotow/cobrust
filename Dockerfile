FROM rust:1.66-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

#------------

FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/cobrust /cobrust
COPY src/public /public

ENTRYPOINT ["/cobrust"]