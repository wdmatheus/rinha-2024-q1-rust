FROM rust:1-slim-buster AS build

WORKDIR /app

COPY ./src ./src
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build --release

FROM debian:buster-slim

COPY --from=build /app/target/release/rinha /app/rinha

CMD "/app/rinha"