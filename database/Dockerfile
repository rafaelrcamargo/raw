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
COPY --from=build /usr/src/rinha/target/x86_64-unknown-linux-musl/release/database .

CMD ["./database"]
