FROM rust:1.86 AS builder

RUN apt-get update
RUN apt-get install -y git jq
RUN rustup component add rust-src

RUN wget -c https://github.com/WebAssembly/binaryen/releases/download/version_119/binaryen-version_119-x86_64-linux.tar.gz -O - | tar -xz -C .
RUN cp binaryen-version_119/bin/wasm-opt /usr/bin/
RUN cargo install sails-cli@0.8.0

WORKDIR /app
