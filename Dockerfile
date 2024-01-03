FROM rust:1.70-buster as builder
COPY . /bitacora
WORKDIR /bitacora
RUN apt-get update && apt-get install -y wget && wget https://github.com/ethereum/solidity/releases/download/v0.8.23/solc-static-linux && chmod +x solc-static-linux
RUN cargo build --release

FROM debian:buster
COPY --from=builder /bitacora/target/release/bitacora /app/bitacora
COPY --from=builder /bitacora/solc-static-linux /usr/local/bin/solc
COPY ./contracts /app/contracts
ENTRYPOINT [ "/app/bitacora" ]