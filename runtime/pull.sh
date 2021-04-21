#!/bin/bash
cd ../compute
git pull

cd ../runtime
git pull

cd ../redismodule
git pull

cd ../compute
cargo build --release
chmod +x build.sh
./build.sh

cd ../redismodule
cargo build --release
chmod +x build.sh
./build.sh

