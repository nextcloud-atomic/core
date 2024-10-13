FROM rust:1-alpine AS builder
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
ENV OPENSSL_NO_VENDOR=y

# Rust packages
RUN apk add bash alpine-sdk coreutils gcc g++ openssl openssl-dev musl-dev clang openssl-libs-static
RUN rustup target add wasm32-unknown-unknown && cargo install dioxus-cli

#RUN cargo update -p wasm-bindgen --precise 0.2.93

# Application environment
RUN mkdir /app
WORKDIR /app

FROM builder AS devcontainer

# SSH
RUN apk update && apk add openssh
COPY devel/ssh_key.pub /root/.ssh/authorized_keys
RUN chmod 0600 /root/.ssh/authorized_keys

COPY resource /resource

COPY devel/entrypoint.sh /entrypoint.sh
CMD ["/entrypoint.sh"]

FROM builder AS app-builder
COPY . /app
RUN --mount=type=cache,target=/app/target \
    dx build --platform fullstack --release --target=x86_64-unknown-linux-musl

FROM alpine:latest AS release

ENV NCA_CONFIG_SOURCE=/resource/templates
ENV NCA_CONFIG_TARGET=/etc/ncatomic
ENV CADDY_ADMIN_SOCKET=/run/caddy/caddy-admin.sock
ENV DIOXUS_IP="0.0.0.0"

COPY --from=app-builder /app/dist /app/dist
COPY resource /resource
EXPOSE 8080
WORKDIR /app
CMD ["/app/dist/activate"]