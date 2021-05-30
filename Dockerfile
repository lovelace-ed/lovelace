FROM rust:1.52 as cargo-build

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/main"]
