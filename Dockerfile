#
# A Dockerized build for the tool.
#

# We build on host platform and cross-compile to target arch
FROM --platform=$BUILDPLATFORM rust:1.63-slim-bullseye as cross
ARG TARGETARCH
COPY docker/platform.sh .
RUN ./platform.sh # writes /.platform and /.compiler
RUN rustup target add $(cat /.platform)

# We do a trick to cache the dependencies in Docker:
#
# 1. Initialize a new, dummy project
# 2. Give it our library dependencies
# 3. Build it so it compiles just those
#
RUN USER=root cargo new --bin json2jsonl
WORKDIR /json2jsonl
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# And now we build our source for real:
RUN rm src/*.rs ./target/release/deps/json2jsonl*
COPY ./src ./src
RUN cargo build --release


FROM debian:bullseye-slim
COPY --from=cross \
    /json2jsonl/target/release/json2jsonl \
    /usr/local/bin/json2jsonl
CMD ["json2jsonl"]