FROM rust:1.72 as builder

#RUN apk add alpine-sdk openssl-dev musl-dev openssl
#RUN apt-get install -y openssl-dev musl-dev
RUN apt-get update && apt-get install -y clang
#ENV OPENSSL_DIR=/usr
RUN rustup target add wasm32-unknown-unknown && cargo install dioxus-cli
WORKDIR /usr/src/ncp-activation
COPY . .
RUN dx build --features web --release && dx build --features ssr --release --platform desktop
#CMD ["trunk", "serve", "--address", "0.0.0.0", "--port", "${PORT}", "--features", "ssr", "--release", ""]
#CMD ["cargo", "run", "--bin", "ncp-activation", "--features", "ssr"]

FROM debian:latest

ENV NCP_CONFIG_SOURCE=/resource/templates
ENV NCP_CONFIG_TARGET=/etc/ncp
ENV CADDY_ADDRESS=unix:/run/caddy-admin.sock

RUN apt-get update && apt-get install -y libssl-dev

COPY resource /resource
COPY --from=builder /usr/src/ncp-activation/dist /dist
CMD ["/dist/ncp-activation"]
