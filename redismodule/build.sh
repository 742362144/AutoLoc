#!/bin/bash
#git clone https://github.com/742362144/runtime.git
#git clone https://github.com/742362144/redismodule.git

rustup toolchain install nightly-2020-12-14
rustup default nightly-2020-12-14-x86_64-unknown-linux-gnu

cargo build --release
cp -f ./target/release/libredismodule.so docker
cp -f redis.conf docker

cd docker
docker build -t storageloc .
#docker run -d -p 6379:6379 storageloc