# Ink Staking

Ink Staking is a smart contract developed using `ink!`, the smart contract language for the Polkadot ecosystem. This project aims to provide a decentralized staking mechanism for token holders, allowing them to earn rewards by participating in the network's staking protocol.

## Features
Allows users to stake their tokens to support the network and receive staking rewards.


## Installation

To set up the `ink-staking` project locally, follow these steps:

1. **Install Rust**: Ensure that you have the latest version of Rust installed. You can install Rust using `rustup`:
    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2. **Add the `wasm32` target**: Add the WebAssembly target required for ink! smart contracts:
    ```sh
    rustup target add wasm32-unknown-unknown
    ```
3. **Clone the Repository**:
    ```sh
    git clone https://github.com/CocDap/ink-staking.git
    cd ink-staking
    ```

4. **Install `rust-src`**:

```bash
rustup component add rust-src
```

5. **Ink! CLI**

```bash
cargo install --force --locked cargo-contract
```

4. **Build the Smart Contract PSP22 Token and Staking **:
```bash
./scripts/build.sh
```

## Usage

To use the staking smart contract:

1. Deploy the compiled `.contract` file to a Substrate-based blockchain that supports `ink!` contracts 


2. Once deployed, users can interact with the staking contract to stake their tokens and earn rewards.

By using particular support tool:
+ Contract UI: https://ui.use.ink/
+ Polkadot JS Explorer: https://polkadot.js.org/apps/
+ Ink! CLI: https://github.com/use-ink/cargo-contract
+ POP CLI: https://github.com/r0gue-io/pop-cli


## Smart Contract Details

- **Contract Language**: `ink!`
- **Platform**: Substrate-based blockchain with `pallet-contracts`

### Functions

- **Stake Tokens**: Users can stake their tokens by calling the `stake` function.
- **Withdraw Tokens**: Users can withdraw their staked tokens using the `withdraw` function.
- **Claim Rewards**: Stakers can claim their accumulated rewards using the `claim_rewards` function.






