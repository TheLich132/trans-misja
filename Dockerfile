FROM fedora:latest
WORKDIR /app
RUN dnf install -y bash rustup gtk4-devel gcc git g++ libadwaita-devel
RUN rustup-init -y
RUN rustup default stable
ENV PATH="/root/.cargo/bin:${PATH}"
RUN source $HOME/.cargo/env
SHELL ["/bin/bash", "-c"]