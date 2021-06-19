FROM rust:1.53 as cargo-build

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/main"]
