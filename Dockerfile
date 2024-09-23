FROM rust:1-alpine as builder
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
ENV OPENSSL_NO_VENDOR=y
RUN apk update && apk add alpine-sdk coreutils gcc g++ openssl openssl-dev musl-dev clang openssl-libs-static
RUN rustup target add wasm32-unknown-unknown && cargo install dioxus-cli
#RUN cargo update -p wasm-bindgen --precise 0.2.93
WORKDIR /app
COPY . .
RUN dx build --platform fullstack --release --target=x86_64-unknown-linux-musl

FROM alpine:latest

ENV NCP_CONFIG_SOURCE=/resource/templates
ENV NCP_CONFIG_TARGET=/etc/ncp
ENV CADDY_ADMIN_SOCKET=/run/caddy/caddy-admin.sock
ENV DIOXUS_IP="0.0.0.0"

COPY --from=builder /app/dist /dist
COPY resource /resource
EXPOSE 8080
WORKDIR /dist
CMD ["/dist/activate"]