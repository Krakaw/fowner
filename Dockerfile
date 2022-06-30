FROM rust:1.61.0-bullseye as builder
WORKDIR /usr/src/fowner

COPY ./api/Cargo.lock .
COPY ./api/Cargo.toml .
RUN echo "pub fn main() {}" >> dummy.rs \
    && sed -i 's#src/main.rs#dummy.rs#' Cargo.toml \
    && cargo build --release \
    && sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY api/src src
RUN cargo build  --release


FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libsqlite3-dev curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/fowner/target/release/fowner /usr/local/bin/fowner
WORKDIR /opt/fowner
EXPOSE 8080
