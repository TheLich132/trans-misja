FROM fedora:latest
WORKDIR /app
RUN dnf install -y rustup gtk4-devel gcc git
RUN rustup-init -y