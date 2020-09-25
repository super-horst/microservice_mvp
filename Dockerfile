FROM rust:1.46.0 as builder

ARG PROJECT_NAME
ENV RUST_BACKTRACE=full

# Compiler
RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends musl-tools

# Tooling
RUN rustup target add x86_64-unknown-linux-musl && \
    rustup component add rustfmt

# Dummy project
RUN USER=root cargo new --bin $PROJECT_NAME
WORKDIR /$PROJECT_NAME

# Build dependencies & generated code for caching
COPY Cargo.toml rust-toolchain build.rs ./
COPY proto ./proto
RUN cargo build --target x86_64-unknown-linux-musl --release

# Copy the source and build the application.
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

# Assemble final container
FROM scratch
EXPOSE 8080

ARG PROJECT_NAME
ENV RUST_BACKTRACE=full
ENV CONFIG=config

COPY --from=builder /usr/local/cargo/bin/$PROJECT_NAME ./service

USER 1000
CMD [ "./service" ]
