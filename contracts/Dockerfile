FROM node:20
COPY . /contracts
WORKDIR /contracts
RUN npm install && npx hardhat compile
CMD [ "npx", "hardhat", "run", "--network", "devnet", "scripts/deploy.ts" ]