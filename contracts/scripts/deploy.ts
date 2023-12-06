import { ethers } from "hardhat";

async function main() {

  const bitacora = await ethers.deployContract("Bitacora");

  await bitacora.waitForDeployment();

  console.log(`Bitacora deployed at ${bitacora.target}`);

  await bitacora.registerDevice("First device", "0x0000000000000000000000000000000000000000000000000000000000123456")
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
