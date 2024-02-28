FROM rust:slim-buster as build
WORKDIR /usr/src/rinha

# Workspace setup
COPY . .

# Build for release
RUN cargo build --release

# Base image
FROM debian:buster-slim
# Update package list
RUN apt-get update && rm -rf /var/lib/apt/lists/*
# Copy build artifact from build stage to this stage
COPY --from=build /usr/src/rinha/target/release/loadbalancer .

CMD ["./loadbalancer"]
