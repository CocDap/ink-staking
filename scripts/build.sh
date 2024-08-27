#!/bin/bash

# Build psp22 
cargo contract build --manifest-path psp22/Cargo.toml

# Build staking contract 
cargo contract build --manifest-path staking/Cargo.toml

