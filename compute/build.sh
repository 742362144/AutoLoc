#!/bin/bash
rustup toolchain install nightly-2020-12-14
rustup default nightly-2020-12-14-x86_64-unknown-linux-gnu

cargo build --release
docker build -t compute .