import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";

const config: HardhatUserConfig = {
  solidity: "0.8.19",
  networks: {
    devnet: {
      url: "http://web3:8545",
      accounts: ["0xa8112819434042a710ecc16997caaa0347369969c2e562ac204a24d9be000259"]
    }
  }
};

export default config;
