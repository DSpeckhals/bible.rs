FROM rust:latest as build

WORKDIR /usr/src/biblers

# Build dependencies only for Rust crate
RUN USER=root cargo new --bin biblers
COPY ./Cargo.lock ./Cargo.toml ./
COPY ./cli/Cargo.toml ./cli/Cargo.toml
COPY ./db/Cargo.toml ./db/Cargo.toml
COPY ./web/Cargo.toml ./web/Cargo.toml
RUN mkdir cli/src \
    && echo "fn main() {}" > cli/src/main.rs \
    && mkdir db/src \
    && echo "fn main() {}" > db/src/main.rs \
    && mkdir web/src \
    && echo "fn main() {}" > web/src/main.rs
RUN cargo build --release -p web \
    && rm -rf db web

# Build the crate
COPY ./db ./db
COPY ./web ./web
RUN cargo build --release -p web

# Build SASS
RUN apt-get update \
    && apt-get -y install sassc \
    && rm -rf /var/lib/apt/lists/*
COPY ./web/styles ./web/styles
RUN sassc web/styles/index.scss web/dist/css/style.css

FROM debian:stretch-slim

RUN apt-get update \
    && apt-get -y install libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root

# Copy built rust binary
COPY --from=build /usr/src/biblers/target/release/web ./biblers

# Copy dist and built SASS
COPY ./web/dist ./web/dist
COPY --from=build /usr/src/biblers/web/dist/css ./dist/css

# Copy migrations and templates
COPY ./db/migrations/ ./db/migrations/
COPY ./web/templates/ ./web/templates/

# Set database variable
ENV DATABASE_URL="/root/bible.db"

EXPOSE 8080
CMD ["./biblers"]
