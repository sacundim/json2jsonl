#
# Build a Docker image for our app.
#
# Is capable of building a multi-architecture image, and uses
# Rust cross-compilation support to compile the foreign architecture
# natively in the host that builds the image.
#
FROM --platform=$BUILDPLATFORM rust:1.63-slim-bullseye AS build
ARG TARGETARCH
COPY docker/platform.sh .
RUN /platform.sh # writes /.platform and /.compiler
RUN apt-get update && apt-get install -y $(cat /.compiler)
RUN rustup target add $(cat /.platform)

# We do a trick to cache the dependencies in Docker:
#
# 1. Initialize a new, dummy project
# 2. Give it our library dependencies
# 3. Build it so it compiles just those
#
RUN cargo new --bin json2jsonl
WORKDIR /json2jsonl
COPY Cargo.toml Cargo.lock ./
COPY docker/cargo-config.toml .cargo/config.toml
RUN cargo build --release --target $(cat /.platform)

# And now we build our source for real:
RUN rm src/*.rs ./target/$(cat /.platform)/release/deps/json2jsonl*
COPY ./src ./src
RUN cargo build --release --target $(cat /.platform)
RUN mv ./target/$(cat /.platform)/release/json2jsonl \
       ./target/release/json2jsonl

FROM debian:bullseye-slim
COPY --from=build \
    /json2jsonl/target/release/json2jsonl \
    /usr/local/bin/json2jsonl
CMD ["json2jsonl"]