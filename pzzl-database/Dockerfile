FROM rust:1.77-buster as builder
WORKDIR /build
RUN apt update && apt install -y musl-tools musl-dev
RUN apt-get install -y build-essential gcc-x86-64-linux-gnu
RUN rustup target add x86_64-unknown-linux-musl
COPY . . 
ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
RUN cargo build --release --target x86_64-unknown-linux-musl

# copy artifacts to a clean image
FROM public.ecr.aws/lambda/provided:al2
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/database /bootstrap
ENTRYPOINT ["/bootstrap"]
