FROM rust:1.80-slim as build

# Copy source code into the build container
WORKDIR /usr/src 
COPY ./ /usr/src

# Install apt packages required for building the package dependencies
RUN apt-get update \
 && apt-get install -y --no-install-recommends --no-install-suggests \
    libssl-dev \
    perl \
    make

# Build Komac from the source code
# RUN cargo build --release --locked
RUN cargo install komac --locked

# Create a new container for
FROM debian:bookworm-slim as release 
RUN apt-get update \
 && apt-get install -y --no-install-recommends --no-install-suggests \
    ca-certificates
COPY --from=build /usr/local/cargo/bin/komac /usr/local/bin/
# COPY --from=build /usr/src/target/release/komac /usr/local/bin/