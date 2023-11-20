# Build stage
FROM rustlang/rust:nightly-slim AS builder
USER 0:0
WORKDIR /home/rust/src

RUN USER=root cargo new --bin winter
WORKDIR /home/rust/src/winter
COPY Cargo.toml Cargo.lock .
COPY src ./src
RUN apt-get update && apt-get install -y libssl-dev pkg-config && cargo install --locked --path .

# Bundle stage
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates ffmpeg
COPY --from=builder /usr/local/cargo/bin/winter .
EXPOSE 3000
ENV WINTER_HOST 0.0.0.0:3000
COPY Winter.toml .
CMD ["./winter"]
