FROM rustlang/rust:nightly AS development

RUN apt-get update && \
    rustup toolchain remove nightly && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup update

RUN cargo install cargo-watch

RUN mkdir /app/
WORKDIR /app

# Copy over your manifests
COPY api/Cargo.toml Cargo.toml

# This build step will cache your dependencies
RUN mkdir /app/src && \
    echo "pub fn main() {}" > ./src/main.rs && \
    cargo build && \
    rm ./src/main.rs
