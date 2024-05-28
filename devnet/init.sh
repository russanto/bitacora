#!/bin/sh
geth init --datadir /root/validator /root/validator/genesis.json
geth --datadir /root/validator \
    --networkid 290 \
    --http --http.api eth,net,web3 --http.addr 0.0.0.0 --http.corsdomain '*' --http.vhosts '*' \
    --nodiscover \
    --mine --miner.etherbase 0x60f2c193181490aa621249f9185717d58f3347da \
    --unlock 0x60f2c193181490aa621249f9185717d58f3347da --password /root/validator/password.txt --allow-insecure-unlock 