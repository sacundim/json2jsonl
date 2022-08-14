#
# A Dockerized build for the tool.
#

FROM rust:1.63-slim-bullseye AS build

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
COPY --from=build /json2jsonl/target/release/json2jsonl /
CMD ["/json2jsonl"]