FROM rust:1.61.0-bullseye as be_builder
WORKDIR /usr/src/fowner

COPY ./api/Cargo.lock .
COPY ./api/Cargo.toml .
RUN echo "pub fn main() {}" >> dummy.rs \
    && sed -i 's#src/main.rs#dummy.rs#' Cargo.toml \
    && cargo build --release \
    && sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY api/src src
RUN cargo build  --release

FROM node:18-bullseye as fe_builder
WORKDIR /usr/src/fowner
COPY ./web/package.json .
COPY ./web/package-lock.json .
RUN npm ci
COPY web .
RUN npm run build

FROM debian:bullseye-slim
WORKDIR /opt/fowner
RUN apt-get update \
    && apt-get install -y openssh-client git libsqlite3-dev curl \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir ~/.ssh && chmod 700 ~/.ssh \
    && printf "Host *\n\tStrictHostKeyChecking no\n\tUserKnownHostsFile /dev/null\n" >> ~/.ssh/config \
    && mkdir /opt/fowner/sources

COPY --from=be_builder /usr/src/fowner/target/release/fowner /usr/local/bin/fowner
COPY --from=fe_builder /usr/src/fowner/build /opt/fowner/public
EXPOSE 8080
