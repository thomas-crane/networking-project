FROM rust:1.42.0-alpine AS builder
WORKDIR /usr/src/networking-project
COPY Cargo.* ./
COPY crates ./crates
RUN ls && \
    cargo build --target x86_64-unknown-linux-musl --release

FROM alpine

# copy the binaries.
COPY --from=builder usr/src/networking-project /usr/src/networking-project

# add the rust binaries to the path and install the `tc` took.
RUN find /usr/src/networking-project/target/x86_64-unknown-linux-musl/release/* \
      -maxdepth 1 \
      -perm /a+x \
      -type f \
      -exec cp {} /usr/bin/ \; && \
    apk add iproute2
