FROM rust:1.70-buster as builder
COPY . /bitacora
WORKDIR /bitacora
RUN cargo build --release

FROM debian:buster
COPY --from=builder /bitacora/target/release/bitacora /app/bitacora
COPY ./contracts /app/contracts
ENTRYPOINT [ "/app/bitacora" ]