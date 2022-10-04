#build stage
FROM rust:alpine3.14 as builder

WORKDIR /home/app

RUN apk add \
    musl-dev \
    cmake \
    g++ \
    make \
    pkgconfig \
    openssl-dev

COPY response-proxy ./response-proxy
COPY common ./common
COPY kafka ./kafka
WORKDIR /home/app/response-proxy
# https://www.reddit.com/r/rust/comments/waoa8a/simple_reqwest_code_is_segfaulting_in_wsl_alpine/
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --release --bin response-proxy
RUN strip target/release/response-proxy

# second stage.
FROM alpine:3.14
WORKDIR /home/app
RUN apk add gcc
COPY --from=builder /home/app/response-proxy/target/release/response-proxy .
# COPY --from=builder /lib/libssl.so.1.1 /lib/libssl.so.1.1
ENTRYPOINT ["./response-proxy"]
