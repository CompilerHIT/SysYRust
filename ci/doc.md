### ci使用前置(手动版)

docker 本地registry部署在局域网中，未配置https,所以需要切换使用http client来允许docker进行不安全的pull

```
sudo touch /etc/docker/daemon.json

sudo echo '{ "insecure-registries":["10.249.12.83:5000"] }' > /etc/docker/daemon.json

sudo systemctl restart docker
```

然后登录该局域docker registry

```
docker login docker login 10.249.12.83:5000 -u root -p root
```

然后拉取最新镜像,

```
docker pull riscv-ci:3.0
```

运行且挂载文件

```
docker 
```

### ci使用初始化(自动版)

使用默认配置初始化ci

```
```

### ci使用流程

如果代码发生了更新，使用-refresh参数指定使用更新后的代码编译程序并且使用新的编译程序,(其中<test_folder1>等是指定的容器中/test/data下要测试的用例文件夹名):

```

test -refresh <test_folder1> <test_folder2> ..
```

如果代码没有发生更新,要使用过去编译的compiler进行编译

```

test <test_folder1> <test_folder2> ...
```

如果要测试容器中挂载到的 /test/下所有用例

```
```