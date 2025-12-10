FROM fedora:latest

WORKDIR /app

RUN dnf update -y && dnf install -y gcc gcc-c++ gtk4-devel libadwaita-devel && dnf clean all

ENV PATH="/root/.cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

CMD ["/bin/bash"]
