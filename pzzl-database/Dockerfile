FROM rust:1.77 AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    musl-tools \
    && apt-get clean

# Create a new user and switch to it
RUN useradd -m rustuser
USER rustuser
WORKDIR /home/rustuser

# Copy the project files
COPY --chown=rustuser:rustuser . .

# Build the project for ARM architecture
RUN rustup target add aarch64-unknown-linux-musl
RUN cargo build --release --target aarch64-unknown-linux-musl

# Second stage: create a small image with the compiled binary
FROM arm64v8/alpine:latest

# Install dependencies
RUN apk --no-cache add ca-certificates

# Copy the compiled binary from the builder stage
COPY --from=builder /home/rustuser/target/aarch64-unknown-linux-musl/release/database /usr/local/bin/my_lambda

# Command to run the binary
CMD ["/usr/local/bin/my_lambda"]
