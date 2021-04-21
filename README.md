# AutoLoc
本构件用于支持课题《专业科技资源及服务集成技术》（2017YFB1400804）。

## 项目简介
无服务器计算作为一种新型的云计算编程范式和计算模型，其要求任务具有无状态的特点以实现计算时的高弹性和可伸缩性。任务间常见的通信方式是将要通信的中间数据存储在一个公共的远程存储服务中。然而这种计算模式需要频繁地与数据存储进行交互，数据通过网络进行传输使得计算需要等待，即使使用诸如RDMA等最先进的网络也会产生严重的性能开销。无服务器计算将计算逻辑封装为计算函数的形式，计算函数作为计算和被调度的最小单元，使得计算函数在存储端运行成为了新的可能。事实上，单纯地在计算端或存储端计算都无法得到最优解，这由计算函数自身的计算逻辑和数据访问等特性决定。不同类型的计算函数在计算端和存储端运行有不同的性能表现，由于无服务器计算场景中计算函数的类型是复杂的，这意味着直接识别函数的计算成本是困难的。无服务器计算平台需要解决函数应该在哪里计算的问题，从而优化无服务器计算平台的系统整体吞吐量。<br>
针对上述研究背景，设计并实现了一种面向无服务器计算场景下的计算函数放置优化系统，允许应用程序在运行时将一小部分本地计算逻辑作为函数推到存储中执行。这些函数可以实现更丰富的数据类型和计算逻辑，通过这种方式避免额外的往返请求并减少数据移动，从而提高整个系统的吞吐量。<br>
实验结果表明，计算函数放置优化系统最多可提高系统整体吞吐量4.49倍，同时在云计算环境中使用基于时间序列模型的资源动态分配机制来对计算资源重新分配，使系统拥有更高的资源使用效率。<br>



## 技术架构
系统架构
![image](https://github.com/742362144/AutoLoc/blob/main/img/fig1.png)

函数状态转换
![image](https://github.com/742362144/AutoLoc/blob/main/img/fig2.png)

## 技术特色
系统基于计算函数运行时行为分析的动态放置优化方法，通过对主流的存储系统如Redis进行定制化改造，使其支持计算函数的放置和运行，再对计算函数运行时分析其在存储端和计算端的损失和收益并通过评估来决定计算函数最优的运行位置，最终获得计算函数放置方案。

## 代码结构
runtime目录为函数运行时，compute为计算节点，redismodule为定制化改造的数据存储与redis结合使用。


## 部署方式
安装redis（6.2）和Docker。<br>
安装rust curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh <br>
chmod +x build.sh <br>
bash build.sh

## 使用说明
docker exec -it compute1 batch khop 1 1000
docker exec -it compute1 redisclient khop 1 1000
