FROM rust:1.94-slim AS build

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=build /app/target/release/worker-rs /usr/local/bin/worker-rs
CMD ["worker-rs"]
