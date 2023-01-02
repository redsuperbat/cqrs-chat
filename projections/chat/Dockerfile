FROM rust:latest as builder

WORKDIR /usr/app

COPY . .

RUN cargo build -p chat-projection --release

FROM alpine:latest

COPY --from=builder /usr/app/target/release/chat-projection .

CMD ["./chat-projection"]