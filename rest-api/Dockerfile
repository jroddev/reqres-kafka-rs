FROM rust:alpine3.14 as builder

WORKDIR /home/app

RUN apk add musl-dev

COPY rest-api .
RUN cargo build --release --bin rest-api
RUN ls target/
RUN strip target/release/rest-api

# second stage.
FROM alpine:3.14
WORKDIR /home/app
COPY --from=builder /home/app/target/release/rest-api .
ENTRYPOINT ["./rest-api"]

