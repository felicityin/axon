const { ethers } = require("hardhat");

async function main() {
    const HeaderProviderContract = await ethers.getContractFactory("GetHeader");
    const headerProvider = await HeaderProviderContract.deploy();
    await headerProvider.deployed();

    const blockHash = "0x178c76f655e629986fc25627484b7a049e90ddb12f19939f9b73491e53070cbd";

    const res = await (await headerProvider.testGetHeader(blockHash)).wait();
    console.log("res: %o\n", res);

    const header = await headerProvider.getHeader();
    console.log("header: %o", header);
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
