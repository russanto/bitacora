FROM node:20-buster as contracts
COPY ./contracts /contracts
WORKDIR /contracts
RUN npm install
RUN npx hardhat compile

FROM rust:1.77-buster as builder
COPY --from=contracts /contracts/artifacts/contracts/Bitacora.sol/Bitacora.json /bitacora/contracts/artifacts/contracts/Bitacora.sol/Bitacora.json
COPY . /bitacora
WORKDIR /bitacora
RUN cargo build --release

FROM debian:buster
COPY --from=builder /bitacora/target/release/bitacora /app/bitacora
RUN apt-get update && apt-get install libssl1.1
ENTRYPOINT [ "/app/bitacora" ]