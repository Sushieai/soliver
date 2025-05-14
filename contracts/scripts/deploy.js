const hre = require("hardhat");

async function main() {
  const Vault = await hre.ethers.getContractFactory("Vault");

  // Deploy contract
  const vault = await Vault.deploy();

  // Wait for deployment to finish
  await vault.waitForDeployment();

  // Get address
  const address = await vault.getAddress();

  console.log(`Vault deployed to: ${address}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
