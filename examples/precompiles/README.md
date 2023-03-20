# Description

It is an Ethereum smart contract project created by [hardhat](https://hardhat.org/tutorial).

# Building

## Requirements

- [node.js](https://github.com/nodesource/distributions#debinstall)
- [yarn](https://classic.yarnpkg.com/lang/en/docs/install/#debian-stable)
- [npx](https://manpages.ubuntu.com/manpages/focal/man1/npx.1.html#install)

## Install

```
yarn
```

## Compile

```
npx hardhat compile
```

## Test

### Start Axon
```
git clone git@github.com:axonweb3/axon.git
cargo build
./target/debug/axon -c devtools/chain/config.toml -g devtools/chain/genesis_single_node.json
```

### Run Test
```
npx hardhat run test/get-cell.js --network axon
```
