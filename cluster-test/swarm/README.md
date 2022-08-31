## 集群初始化
配置/etc/hosts/
```
122.70.153.18   host01
122.70.153.19   host02
122.70.153.20   host03
122.70.153.22   host04
122.70.153.21   meta
```
初始化manager节点（即21服务器）  
```
docker swarm init --advertise-addr <MANAGER-IP>
```
执行成功后，该命令将提示如何将worker节点加入swarm集群  
之后可考虑检查下述端口是否正常开启（部署脚本未使用overlay网络，若4789端口未开启，可忽略）
- TCP port 2377 for cluster management communications
- TCP and UDP port 7946 for communication among nodes
- UDP port 4789 for overlay network traffic  

## 集群配置
|Node|	Minio|	Etcd|	Meta|	Compute|	Frontend|	Compact|	Kafka|
|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
|HOST01(18)|Nginx|	--|	--|	y|	--|	y|	--|
|HOST02(19)| 	--|	--|	--|	y|	--|	y|	--|
|HOST03(20)|	y|	y|	--|	y|	--|	y|	--|
|HOST04(22)|	y|	y|	y|	y|	--|	y|	--|
|META(21)|	--|	--|	--|	--|	y|	--|	y|


## 集群启动
- 启动前需将各节点IP信息导入环境变量  
  ```
  sh env.sh
  ```
- ```deploy.sh```可简化重复参数的输入  
  逐一执行启动指令后建议使用```docker stack ps STACK_NAME```逐一检查服务状态，若CURRENT STATE为Running则启动正常。
  - 启动etcd 
    ```
    ./deploy.sh etcd.docker-compose.yml etcd 
    ```
  - 启动minio 
    ```
    ./deploy.sh minio.docker-compose.yml minio
    ```
    该启动脚本依赖nginx配置文件nginx.conf
  - 启动meta 
    ```
    ./deploy.sh meta.docker-compose.yml meta
    ```
  - 启动frontend、compute、compactor 
    ```
    ./deploy.sh fcc.docker-compose.yml fcc
    ```
  - 启动kafka、loadgen、load-view 
    ```
    docker-compose up -d
    ```
  - 连接piestream并查询 
    ```
    docker-compose run psql
    ```
- 停止集群
  - 关闭集群中某一服务 
    ```
    docker stack rm STACK_NAME
    ```
  - 关闭集群中所有服务 
    ```
    docker service rm $(docker service ls -q)
    ```
  - 清理集群中所有未被使用的volume（docker stack/service rm不会删除相应的volume和network）
    ```
    ./clean.sh
    ```

## FAQ
- 某项服务卡在Pending状态
  - 在分配该服务的节点上执行
    ```journalctl -u docker```
    查看docker daemon日志
  - 若日志提示连接不到image registry，可尝试在docker-compose.yml中利用镜像哈希指定镜像版本，如：```minio/minio:latest -> minio/minio@sha256:68a4352cfa1db4b94e2e7ee72aaa93bc0aecadad97ad5ef0cbb2368ab8ea8efe```
- 某项服务异常退出
  - 在manager节点执行
    ```docker service logs SERVICE_NAME```
    查看服务日志
  - 在分配该服务的节点上执行
    ```docker ps -a```找到异常退出的CONTAINER_ID，执行```docker logs CONTAINER_ID```查看服务日志
  - 在manager节点执行```docker service inspect SERVICE_NAME```查看各项参数是否正确
