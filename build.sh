#!/bin/bash

SHELL_FOLDER=$(cd "$(dirname "$0")";pwd)
cd $SHELL_FOLDER

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cd $SHELL_FOLDER"/compute"
pwd
chmod +x build.sh
pwd
bash build.sh

cd $SHELL_FOLDER"/redismodule"
chmod +x build.sh
bash build.sh

docker rm -f storageloc compute1 compute2

docker run -d -it --cpu-period=100000 --cpu-quota=200000 -m 4G --net=host -v /tmp/storageloc/:/tmp/ --name=storageloc storageloc
docker run -d -it --cpu-period=100000 --cpu-quota=100000 -m 2G --net=host -v /tmp/compute1/:/tmp/  -v /tmp/AutoControl/:/root/AutoControl/ --name=compute1 compute
docker run -d -it --cpu-period=100000 --cpu-quota=100000 -m 2G --net=host -v /tmp/compute2/:/tmp/  -v /tmp/AutoControl/:/root/AutoControl/  --name=compute2 compute


cd $SHELL_FOLDER
pip3 install redis
python3 dataloader.py
redis-cli service