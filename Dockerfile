################### SQLite3 Build ###################
FROM debian:stretch-slim as sqlite-build

WORKDIR /root

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get -y install \
        autoconf \
        curl \
        gcc \
        make \
        tcl \
    && rm -rf /var/lib/apt/lists/*

RUN curl -SL https://www.fossil-scm.org/fossil/uv/fossil-linux-x64-2.7.tar.gz \
    | tar -xzC /usr/local/bin \
    && mkdir sqlite sqlite-build \
    && cd sqlite \
    && fossil clone --user=root http://www.sqlite.org/cgi/src sqlite.fossil \
    && fossil open --user=root sqlite.fossil \
    && fossil update --user=root trunk

RUN cd sqlite-build \
    && ../sqlite/configure --enable-fts5 --disable-fts3 --disable-fts4

RUN cd sqlite-build \
    && make libsqlite3.la \
    && make lib_install


################### Rust Build ###################
FROM rust:latest as rust-build

# Clang/LLVM are required for building the libsqlite3-sys bindings
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get -y install \
        clang-3.9 \
        libclang-3.9-dev \
        llvm-3.9-dev \
        sassc \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/biblers

RUN USER=root cargo new --bin biblers

# Copy sqlite3 lib (for bindgen)
COPY --from=sqlite-build /usr/local/lib/libsqlite3.so.0 /usr/lib/x86_64-linux-gnu/

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

RUN cargo build --release \
    && rm -rf db/src web/src

# Build the crate
COPY ./db/src ./db/src
COPY ./web/src ./web/src
RUN cargo build --release -p web

# Build SASS
COPY ./web/styles ./web/styles
RUN mkdir -p web/dist/css \
    && sassc web/styles/index.scss web/dist/css/style.css


################### Server Build ###################
FROM debian:stretch-slim

WORKDIR /root

# Copy sqlite3 lib
COPY --from=sqlite-build /usr/local/lib/libsqlite3.so.0 /usr/lib/x86_64-linux-gnu/

# Copy built rust binary
COPY --from=rust-build /usr/src/biblers/target/release/web ./biblers

# Copy dist and built SASS
COPY ./web/dist ./web/dist
COPY --from=rust-build /usr/src/biblers/web/dist/css/style.css ./web/dist/css/style.css

# Copy migrations and templates
COPY ./db/migrations/ ./db/migrations/
COPY ./web/templates/ ./web/templates/

# Set database variable
ENV DATABASE_URL="/root/bible.db"

EXPOSE 8080
CMD ["./biblers"]
