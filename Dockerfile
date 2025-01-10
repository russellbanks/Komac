FROM rust:1.84-slim as build

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
RUN cargo build --release

# Create a new container for
FROM debian:bookworm-slim as release 
RUN apt-get update \
 && apt-get install -y --no-install-recommends --no-install-suggests \
      ca-certificates \
 && rm -rf  \
      /var/lib/apt/lists/* \
      /tmp/* \
      /var/tmp/*

COPY --from=build /usr/src/target/release/komac /usr/local/bin/
WORKDIR /root