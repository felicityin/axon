const { ethers } = require("hardhat");

async function main() {
    const CellProviderContract = await ethers.getContractFactory("GetCell");
    const cellProvider = await CellProviderContract.deploy();
    await cellProvider.deployed();

    const txHash = "0x178c76f655e629986fc25627484b7a049e90ddb12f19939f9b73491e53070cbd";
    const index = 0;

    const res = await (await cellProvider.testGetCell(txHash, index)).wait();
    console.log("res: %o\n", res);

    const cell = await cellProvider.getCell();
    console.log("cell: %o", cell);
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
