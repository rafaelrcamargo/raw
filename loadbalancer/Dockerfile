FROM rust:slim-bookworm as build
WORKDIR /usr/src/rinha

# Workspace setup
COPY . .

RUN apt-get update -yqq && apt-get install -yqq cmake g++

# Build for release
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

# Base image
FROM debian:bookworm-slim
# Update package list
RUN apt-get update -yqq && rm -rf /var/lib/apt/lists/*
# Copy build artifact from build stage to this stage
COPY --from=build /usr/src/rinha/target/release/loadbalancer .

CMD ["./loadbalancer"]
