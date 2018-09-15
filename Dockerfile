FROM rust:latest as build

WORKDIR /usr/src/receptus

RUN USER=root cargo new --bin receptus

COPY ./Cargo.lock ./Cargo.toml ./

RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/bin/server.rs \
    && echo "fn main() {}" > src/bin/cli.rs

RUN cargo build --release
RUN rm src/bin/*.rs

COPY ./src ./src

RUN cargo build --release --bin server

FROM debian:stretch-slim

RUN apt-get update \
    && apt-get -y install libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root

COPY --from=build /usr/src/receptus/target/release/server ./server
COPY ./migrations/ ./migrations/
COPY ./templates/ ./templates/
COPY ./static/ ./static/

ENV DATABASE_URL="/root/bible.db"

EXPOSE 8080
CMD ["./server"]
