# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
FROM rust:1.46.0 as builder

ARG RUST_BACKTRACE=full

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
WORKDIR /usr/src/microservice
RUN USER=root cargo init
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Copy the source and build the application.
COPY src ./src
RUN cargo install --path .

# Copy the statically-linked binary into a scratch container.
FROM debian:stable-slim
COPY --from=builder /usr/local/cargo/bin/microservice .
USER 1000
CMD ["./microservice"]
