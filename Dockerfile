FROM ekidd/rust-musl-builder:latest AS builder
ADD --chown=rust:rust . ./
RUN cargo build --release

FROM alpine:latest
WORKDIR app
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/twitter-exporter /app
CMD ["/app/twitter-exporter"]
