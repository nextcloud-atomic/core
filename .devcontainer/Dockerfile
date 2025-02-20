FROM purtontech/rust-on-nails-devcontainer:1.3.15 AS development
ENV BINSTALL_NO_CONFIRM=yes
RUN sudo apt update && sudo apt upgrade -y && sudo apt install -y --no-install-recommends libsystemd-dev pkg-config inotify-tools

COPY ps1.bash .
RUN cat ps1.bash >> ~/.bashrc && sudo rm ps1.bash

COPY .bash_aliases /home/vscode/.bash_aliases

# Enable our git hooks and set the permisisons on docker sock.
RUN echo 'git config core.hooksPath /workspace/.devcontainer/.githooks' >> ~/.bashrc

# all the volumes configured in the docker-compose.yml
ARG USERNAME=vscode
RUN rustup update && rustup default stable && rustup component add clippy rustfmt && cargo install cargo-update && cargo install-update -a
RUN cargo binstall dioxus-cli || cargo install dioxus-cli
COPY dummy-logstream /usr/local/bin/dummy-logstream
RUN sudo mkdir -p /workspace/target && sudo chown $USERNAME:$USERNAME /workspace/target
RUN sudo mkdir -p /workspace/crates/nca-frontend/node_modules && sudo chown $USERNAME:$USERNAME /workspace/crates/nca-frontend/node_modules
ENTRYPOINT /usr/bin/bash
CMD ["-c", "/usr/local/bin/dummy-logstream"]
