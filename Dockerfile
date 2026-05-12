################### Rust Build ###################
FROM docker.io/rust:trixie AS rust-build

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get -y install sassc && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/biblers

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./cli/Cargo.toml ./cli/Cargo.toml
COPY ./db/Cargo.toml ./db/Cargo.toml
COPY ./web/Cargo.toml ./web/Cargo.toml

RUN mkdir cli/src && echo "fn main() {}" > cli/src/main.rs && \
    mkdir db/src && echo "fn main() {}" > db/src/main.rs && \
    mkdir web/src && echo "fn main() {}" > web/src/main.rs

RUN cargo build --release && rm -rf db/src web/src

COPY ./db/src ./db/src
COPY ./web/src ./web/src
RUN cargo build --release -p web

COPY ./web/styles ./web/styles
RUN mkdir -p web/dist/css && \
    sassc web/styles/index.scss web/dist/css/style.css


################### Server Build ###################
FROM docker.io/debian:trixie-slim

WORKDIR /app

COPY --from=rust-build /usr/src/biblers/target/release/web ./biblers
COPY ./web/dist ./web/dist
COPY --from=rust-build /usr/src/biblers/web/dist/css/style.css ./web/dist/css/style.css
COPY ./db/migrations/ ./db/migrations/
COPY ./web/templates/ ./web/templates/

RUN mkdir -p /opt/data

EXPOSE 8080
CMD ["./biblers"]
