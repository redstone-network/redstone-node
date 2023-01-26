[![Substrate version](https://img.shields.io/badge/Substrate-3.0.0-blue?logo=Parity%20Substrate)](https://substrate.dev/) [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](#LICENSE)


<a href='https://web3.foundation/'><img width='205' alt='web3f_grants_badge.png' src='https://github.com/heyworld88/gitskills/blob/main/web3f_grants_badge.png'></a>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<a href='https://builders.parity.io/'><img width='240' src='https://github.com/heyworld88/gitskills/blob/main/sbp_grants_badge.png'></a>

  
**The Redstone Network is a network of trigger circuits where users combine and arrange simple atomic trigger components according to certain rules and processes to eventually implement a series of automated operational circuits. We propose transaction firewall middleware that can function between any blockchain network, regardless of the cross-chain mechanism and network topology used to execute asset transfers, and regardless of the type of assets traded, we will construct a firewall of different security levels for users. With the help of triggers and machine learning technologies, we will provide users with passive defense and proactive alerting capabilities.** 

## Getting Started


### Install Guide

Follow [Setup](https://docs.substrate.io/install/macos/) to guide you install the RedStone development.

### Build Node

The `cargo run` command will perform an initial build. Use the following command to build the node without launching it:

```
# Fetch the code
git clone https://github.com/redstone-network/redstone-node.git
cd redstone-node

# Build the node (The first build will be long (~30min))
cargo build --release
```

## Run The RSN Node


After the node has finished compiling, you can follow these steps below to run it. 

### Generate Keys

If you already have keys for Substrate using the [SS58 address encoding format](https://docs.substrate.io/v3/advanced/ss58/), please see the next section.

Begin by compiling and installing the utility ([instructions and more info here](https://substrate.dev/docs/en/knowledgebase/integrate/subkey)). 

Generate a mnemonic (Secret phrase) and see the `sr25519` key and address associated with it.

```
# subkey command
subkey generate --scheme sr25519
```

Now see the `ed25519` key and address associated with the same mnemonic (secret phrase).

```
# subkey command
subkey inspect --scheme ed25519 "SECRET PHRASE YOU JUST GENERATED"
```

We recommend that you record the above outputs and keep mnemonic in safe.

### Run Testnet

Launch node on the redstone-testnet with:

```
# start
./target/release/redstone-node --base-path /tmp/redstone --chain dev --alice
```

## Run in Docker

Install Docker first, then run the following command to start a node, and finally access it through port 9944:

```
docker pull baidang201/redstone-node:latest
docker run -d --network host baidang201/redstone-node:latest /usr/local/bin/redstone-node --dev --rpc-cors all --rpc-methods=unsafe --rpc-external --unsafe-ws-external --unsafe-rpc-external
```

## Run Tests


RedStone has Rust unit tests, and can be run locally.

```
# Run all the Rust unit tests
cargo test --release
```

## Module Documentation


* [Defense Module](https://github.com/redstone-network/redstone-node/tree/main/pallets/defense)
* [Notification Module](https://github.com/redstone-network/redstone-node/tree/main/pallets/notification)
* [Permission-capture Module](https://github.com/redstone-network/redstone-node/tree/main/pallets/permission-capture)

## redstone-bootstrap

Please download the code of the current latest release
```
cd redstone-node
#Under the /redstone root directory
cargo build --release
```
Go to the /redstone-node/target/release directory to obtain the redstone node file
