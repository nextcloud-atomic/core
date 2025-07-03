# TODO: Move to official rust:trixie-slim image once available
FROM debian:trixie-slim AS rust-base

LABEL org.opencontainers.image.source=https://github.com/rust-lang/docker-rust

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.88.0

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        ; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='3b8daab6cc3135f2cd4b12919559e6adaee73a2fbefb830fadf0405c20231d61' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='e3853c5a252fca15252d07cb23a1bdd9377a8c6f3efa01531109281ae47f841c' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='a5db2c4b29d23e9b318b955dd0337d6b52e93933608469085c924e0d05b1df1f' ;; \
        ppc64el) rustArch='powerpc64le-unknown-linux-gnu'; rustupSha256='acd89c42b47c93bd4266163a7b05d3f26287d5148413c0d47b2e8a7aa67c9dc0' ;; \
        s390x) rustArch='s390x-unknown-linux-gnu'; rustupSha256='726b7fd5d8805e73eab4a024a2889f8859d5a44e36041abac0a2436a52d42572' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.28.2/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*;

# Do all the cargo install stuff
FROM rust-base AS builder

ARG TARGETOS
ARG TARGETARCH
ARG TARGETPLATFORM

ARG CARGO_BINSTALL_VERSION=0.8.0
ARG CARGO_WATCH_VERSION=7.8.0

# Configure apt and install packages
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libpq-dev \
        curl \
        wget \
        xz-utils \
        build-essential \
        unzip

# Install cargo-watch
RUN case "${TARGETARCH}" in \
        amd64) ARCH_WATCH="x86_64-unknown-linux-musl" ;; \
        arm64) ARCH_WATCH="aarch64-unknown-linux-gnu" ;; \
        *) echo "Unsupported architecture for cargo-watch: ${TARGETARCH}" && exit 1 ;; \
    esac \
    && curl -L https://github.com/watchexec/cargo-watch/releases/download/v${CARGO_WATCH_VERSION}/cargo-watch-v${CARGO_WATCH_VERSION}-${ARCH_WATCH}.tar.xz -o cargo-watch.tar.xz \
    && tar -xf cargo-watch.tar.xz \
    && mv cargo-watch-v${CARGO_WATCH_VERSION}-${ARCH_WATCH}/cargo-watch /home/ \
    && chmod +x /home/cargo-watch \
    && rm -rf cargo-watch.tar.xz cargo-watch-v${CARGO_WATCH_VERSION}-${ARCH_WATCH}

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
# Determine architecture and install cargo-binstall
RUN case "${TARGETARCH}" in \
        amd64) ARCH="x86_64-unknown-linux-musl" ;; \
        arm64) ARCH="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac \
    && wget https://github.com/cargo-bins/cargo-binstall/releases/download/v${CARGO_BINSTALL_VERSION}/cargo-binstall-${ARCH}.tgz \
    && tar -xvf cargo-binstall-${ARCH}.tgz \
    && cp cargo-binstall /usr/local/cargo/bin/ \
    && rm cargo-binstall-${ARCH}.tgz

FROM rust-base AS devcontainer-base

ARG TARGETOS
ARG TARGETARCH
ARG TARGETPLATFORM

ARG CLOAK_VERSION=1.19.4
ARG ZOLA_VERSION=0.17.2
ARG DBMATE_VERSION=2.26.0
ARG MOLD_VERSION=2.34.1
ARG EARTHLY_VERSION=0.8.15
ARG DOCKER_COMPOSE_VERSION=2.23.0
ARG POSTGRES_CLIENT_VERSION=17


# This Dockerfile adds a non-root 'vscode' user with sudo access. However, for Linux,
# this user's GID/UID must match your local user UID/GID to avoid permission issues
# with bind mounts. Update USER_UID / USER_GID if yours is not 1000. See
# https://aka.ms/vscode-remote/containers/non-root-user for details.
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Configure apt and install packages
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        ca-certificates \
        gnupg \
        gnupg-agent \
        curl \
        postgresql-common && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        apt-transport-https \
        build-essential \
        git \
        jq \
        musl \
        musl-dev \
        musl-tools \
        nodejs \
        npm \
        protobuf-compiler \
        ssh \
        sudo \
        wget \
        postgresql-client-${POSTGRES_CLIENT_VERSION} && \
    apt-get autoremove -y && \
    apt-get clean -y && \
    rm -rf /var/lib/apt/lists/* /var/cache/* \
    # Docker Engine for Earthly. https://docs.docker.com/engine/install/debian/
    && install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg \
    && chmod a+r /etc/apt/keyrings/docker.gpg \
    && echo \
        "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
        "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
        tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get -y update \
    && apt-get -y --no-install-recommends install docker-ce docker-ce-cli containerd.io \
    && apt-get autoremove -y && apt-get clean -y \
    # Tailwind binary + extra
#    && curl -OL https://github.com/dobicinaitis/tailwind-cli-extra/releases/latest/download/tailwindcss-extra-linux-x64 \
#    && chmod +x tailwindcss-extra-linux-x64 \
#    && mv tailwindcss-extra-linux-x64 /usr/local/bin/tailwind-extra \

    # Create a non-root user
    && groupadd --gid $USER_GID $USERNAME \
    && useradd -s /bin/bash --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME\
    && chmod 0440 /etc/sudoers.d/$USERNAME \
    # Rust tools
    && rustup component add rustfmt clippy \
    # Add the musl toolchain
    && rustup target add x86_64-unknown-linux-musl \
    && rustup target add aarch64-unknown-linux-musl \
    && rustup target add wasm32-unknown-unknown \
     # Database migrations
    && curl -OL https://github.com/amacneil/dbmate/releases/download/v$DBMATE_VERSION/dbmate-linux-${TARGETARCH} \
    && mv ./dbmate-linux-${TARGETARCH} /usr/bin/dbmate \
    && chmod +x /usr/bin/dbmate \
    # Mold - Fast Rust Linker
    && curl -OL https://github.com/rui314/mold/releases/download/v$MOLD_VERSION/mold-$MOLD_VERSION-x86_64-linux.tar.gz \
    && tar -xf mold-$MOLD_VERSION-x86_64-linux.tar.gz \
    && mv ./mold-$MOLD_VERSION-x86_64-linux/bin/mold /usr/bin/ \
    && mv ./mold-$MOLD_VERSION-x86_64-linux/lib/mold/mold-wrapper.so /usr/bin/ \
    && rm mold-$MOLD_VERSION-x86_64-linux.tar.gz \
    && rm -rf ./mold-$MOLD_VERSION-x86_64-linux \
    && chmod +x /usr/bin/mold

RUN wget https://github.com/earthly/earthly/releases/download/v$EARTHLY_VERSION/earthly-linux-${TARGETARCH} -O /usr/local/bin/earthly \
    && chmod +x /usr/local/bin/earthly \
    # K9s
    && curl -L -s https://github.com/derailed/k9s/releases/download/v0.32.4/k9s_Linux_${TARGETARCH}.tar.gz | tar xvz -C /tmp \
    && mv /tmp/k9s /usr/bin \
    && rm -rf k9s_Linux_*.tar.gz \
    # Kubectl
    && curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/${TARGETARCH}/kubectl" \
    && install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl \
    # Just
    && curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin \
    && mv ~/bin/just /usr/local/bin

USER $USERNAME

# Copy the binaries we built in builder container
COPY --chown=$USERNAME --from=builder /home/cargo-watch $CARGO_HOME/bin
COPY --chown=$USERNAME --from=builder /usr/local/cargo/bin/cargo-binstall $CARGO_HOME/bin


# Pulumi
RUN curl -fsSL https://get.pulumi.com | sudo -E bash - \
    && sudo chown -R $USERNAME:$USERNAME /home/$USERNAME/.pulumi

# Compile here as it requires musl.
RUN cargo binstall --no-confirm cornucopia

RUN if [ "$TARGETARCH" != "arm64" ]; then \
        cargo binstall --no-confirm clorinde; \
    fi


FROM devcontainer-base AS development
ENV BINSTALL_NO_CONFIRM=yes
RUN sudo apt-get update && sudo apt-get upgrade -y && sudo apt-get install -y --no-install-recommends libsystemd-dev pkg-config inotify-tools \
    libwebkit2gtk-4.1-dev libatk1.0-dev libglib2.0-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf-xlib-2.0-dev \
    libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libudev-dev libblkid-dev util-linux libclang-dev pkg-config && sudo apt install -y --install-recommends bash bash-completion zsh

COPY ps1.bash .
RUN cat ps1.bash >> ~/.bashrc && sudo rm ps1.bash

COPY .bash_aliases /home/vscode/.bash_aliases

# Enable our git hooks and set the permisisons on docker sock.
RUN echo 'git config core.hooksPath /workspace/.devcontainer/.githooks' >> ~/.bashrc

# all the volumes configured in the docker-compose.yml
ARG USERNAME=vscode
RUN rustup update && rustup default stable && rustup component add clippy rustfmt && cargo install cargo-update && cargo install-update -a
RUN cargo binstall dioxus-cli || cargo install dioxus-cli
RUN sudo mkdir -p /workspace/target && sudo chown $USERNAME:$USERNAME /workspace/target
RUN sudo mkdir -p /workspace/crates/nca-frontend/node_modules && sudo chown $USERNAME:$USERNAME /workspace/crates/nca-frontend/node_modules
ENTRYPOINT /usr/bin/bash
CMD ["-c", "/usr/local/bin/dummy-logstream"]
