#!/bin/bash
VERSION=`cat .DFX_VERSION`
export PATH=~/.cache/dfinity/versions/$VERSION:`pwd`:$PATH
dfx start --background --clean &&\
dfx canister create mazeGame &&\
dfx build mazeGame &&\
dfx canister install mazeGame ||\
dfx canister install mazeGame --mode=reinstall &&\
cargo run --release -- -d connect 127.0.0.1:8000 `dfx canister id mazeGame`

