FROM rust:latest as build

WORKDIR /usr/src/biblers

# Build dependencies only for Rust crate
RUN USER=root cargo new --bin biblers
COPY ./Cargo.lock ./Cargo.toml ./
RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/bin/server.rs \
    && echo "fn main() {}" > src/bin/cli.rs
RUN cargo build --release
RUN rm src/bin/*.rs

# Build the crate
COPY ./src ./src
RUN cargo build --release --bin server

# Build SASS
RUN apt-get update \
    && apt-get -y install sassc \
    && rm -rf /var/lib/apt/lists/*
COPY ./styles ./styles
RUN mkdir -p dist/css \
    && sassc styles/index.scss dist/css/style.css

FROM debian:stretch-slim

RUN apt-get update \
    && apt-get -y install libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root

# Copy built rust binary
COPY --from=build /usr/src/biblers/target/release/server ./server

# Copy dist and built SASS
COPY ./dist ./dist
COPY --from=build /usr/src/biblers/dist/css ./dist/css

# Copy migrations and templates
COPY ./migrations/ ./migrations/
COPY ./templates/ ./templates/

# Set database variable
ENV DATABASE_URL="/root/bible.db"

EXPOSE 8080
CMD ["./server"]
