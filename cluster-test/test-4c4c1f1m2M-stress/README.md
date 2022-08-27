## 基本情况

在各服务器/home/abc/rise目录下存有节点的 *.docker-compose.yml  
可以各节点手动启动，也可以免密登录21服务器  
18-22服务器均可相互免密登录，各节点hosts文件如下所示

```
122.70.153.18   host01
122.70.153.19   host02
122.70.153.20   host03
122.70.153.22   host04
122.70.153.21   meta
```

## 集群配置

|Node |Minio|Etcd |Meta |Compute|Frontend|Compact|Kafka
|:--: |:--: |:--:|:--:|:--:|:--:|:--:|:--:|
|NODE1(18)|--   |--  |--  |y   |-- |y |--|
|NODE2(19)|--   |--  |--  |y   |-- |y |--|
|NODE3(20)|y    |y   |--  |y   |-- |y |--|
|NODE4(22)|y    |y   |y   |y   |-- |y |--|
|META (21)|Nginx|--  |--  |--  |y  |--|y |


## 管理集群

```bash
# 进入目录
cd test-4c4c1f1m4M
# 推送 *compose.yml 至其他节点
../bin/push-compose.sh
# 启动集群
../bin/up.sh
# 停止集群
../bin/down.sh
# 停止并清空集群数据
../bin/down-all.sh
# 记录 top 数据, 存于 top-logs
../bin/top-start.sh
# 停止 top 数据记录
../bin/stop-stop.sh

# 查看日志: 
# ./logs.sh {HOSTNAME} {SERVICE_NAME}
./logs.sh host01 compute
./logs.sh host03 etcd
./logs.sh meta meta
```
