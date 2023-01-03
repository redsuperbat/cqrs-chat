FROM rust:latest as builder

WORKDIR /usr/app

COPY . .

RUN cargo build -p chat-subscriber --release

FROM ubuntu:latest

WORKDIR /usr/app

COPY --from=builder /usr/app/target/release/chat-subscriber .

CMD ["./chat-subscriber"]