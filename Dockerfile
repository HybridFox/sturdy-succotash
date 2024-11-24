FROM rust:1.82.0 AS builder

# create a new empty shell project
# RUN USER=root cargo new --bin app
WORKDIR /app

# copy over your manifests
COPY . /app/

ENV SQLX_OFFLINE=true
# build for release
RUN cargo build --bins --release

RUN ldd /app/target/release/verkeers-data

# our final base
FROM debian:bookworm-slim

RUN apt-get update && apt-get install libpq5 libssl-dev ca-certificates -y
COPY --from=builder /app/target/release/verkeers-data /

# set the startup command to run your binary
CMD ["./verkeers-data"]
