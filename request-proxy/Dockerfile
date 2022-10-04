FROM rust:alpine3.14 as builder

WORKDIR /home/app

RUN apk add musl-dev cmake g++ make

COPY request-proxy ./request-proxy
COPY common ./common
COPY kafka ./kafka
WORKDIR /home/app/request-proxy
RUN cargo build --release --bin request-proxy
RUN ls target/
RUN strip target/release/request-proxy

# second stage.
FROM alpine:3.14
WORKDIR /home/app
COPY --from=builder /home/app/request-proxy/target/release/request-proxy .
ENTRYPOINT ["./request-proxy"]
