FROM rust:1.77 AS builder

# Install dependencies
RUN apt-get update && apt-get install -y musl-tools curl && apt-get clean

# Create a new user and switch to it
WORKDIR /build

COPY . .
RUN mkdir -p /etc/ssl/certs

# Download the SSL certificate directly in the Dockerfile
RUN curl -o /etc/ssl/certs/rds-ca-cert.pem https://truststore.pki.rds.amazonaws.com/global/global-bundle.pem

# Build the project for ARM architecture
RUN rustup target add aarch64-unknown-linux-musl
RUN cargo build --release --target aarch64-unknown-linux-musl

# Second stage: create a small image with the compiled binary
FROM arm64v8/alpine:latest

# Install dependencies
RUN apk --no-cache add ca-certificates
RUN mkdir -p /etc/ssl/certs

# Copy the compiled binary from the builder stage
COPY --from=builder /build/target/aarch64-unknown-linux-musl/release/database /usr/local/bin/my_lambda
COPY --from=builder /build/migrations /usr/local/bin/
COPY --from=builder /etc/ssl/certs/rds-ca-cert.pem /etc/ssl/certs/rds-ca-cert.pem

ENV DB_CA_CERT=/etc/ssl/certs/rds-ca-cert.pem

# Command to run the binary
CMD ["/usr/local/bin/my_lambda"]
