#
# A Dockerized build for the tool.
#

FROM rust:1.58-slim-bullseye AS build
RUN USER=root cargo new --bin json2jsonl
WORKDIR /json2jsonl

# We do a trick to cache the dependencies:
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

# And now we build for real:
COPY ./src ./src
RUN rm ./target/release/deps/json2jsonl*
RUN cargo build


FROM debian:bullseye-slim
COPY --from=build /json2jsonl/target/release/json2jsonl /
CMD ["/json2jsonl"]