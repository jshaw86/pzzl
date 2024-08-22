FROM rust:1.77 AS builder

# Install dependencies
RUN apt-get update && apt-get install -y musl-tools curl && apt-get clean

# Create a new user and switch to it
WORKDIR /build

COPY . .

# Build the project for ARM architecture
RUN rustup target add aarch64-unknown-linux-musl
RUN cd pzzl-lambda && cargo build --release --target aarch64-unknown-linux-musl

# Second stage: create a small image with the compiled binary
FROM arm64v8/alpine:latest

# Copy the compiled binary from the builder stage
COPY --from=builder /build/pzzl-lambda/target/aarch64-unknown-linux-musl/release/pzzl-lambda /usr/local/bin/my_lambda

# Command to run the binary
CMD ["/usr/local/bin/my_lambda"]
