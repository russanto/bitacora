import {
  time,
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";

describe("Bitacora", function () {
  
	async function deploy() {
		// Contracts are deployed using the first signer/account by default
		const Bitacora = await ethers.getContractFactory("Bitacora");
		const bitacora = await Bitacora.deploy();
		return { bitacora };
  	}

  	describe("Deployment", function () {
		it("Should deploy the contract", async function () {
			await loadFixture(deploy);
		});
  	});
	
});
