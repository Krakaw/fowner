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
RUN apt-get update \
    && apt-get install -y openssh-client git libsqlite3-dev curl \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir ~/.ssh && chmod 700 ~/.ssh \
    && echo -e "Host *\n\tStrictHostKeyChecking no\n\tUserKnownHostsFile /dev/null\n" >> ~/.ssh/config
COPY --from=builder /usr/src/fowner/target/release/fowner /usr/local/bin/fowner
WORKDIR /opt/fowner
EXPOSE 8080
