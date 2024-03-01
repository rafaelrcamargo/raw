FROM rustlang/rust:nightly-alpine as build
WORKDIR /usr/src/rinha

RUN apk update && apk add musl-dev

# Workspace setup
COPY . .

# Build for release
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

# Base image
FROM alpine:latest

# Copy build artifact from build stage to this stage
COPY --from=build /usr/src/rinha/target/release/loadbalancer .

CMD ["./loadbalancer"]
