
### 测试镜像版本:

`10.249.12.83:5000/compilerhit/sysy-rv64-cpci:2.2` :
修复了输出格式与标准输出的区别,

`10.249.12.83:5000/compilerhit/sysy-rv64-cpci:2.1` :
增加目标程序执行时间获取


### ci使用前置(手动版)

1\.docker 本地registry部署在局域网中，未配置https,所以需要切换使用http client来允许docker进行不安全的pull

```
sudo touch /etc/docker/daemon.json

sudo echo '{ "insecure-registries":["10.249.12.83:5000"] }' > /etc/docker/daemon.json

sudo systemctl restart docker
```

2\.然后登录该局域docker registry

```
docker login 10.249.12.83:5000 -u root -p root
```

3\.然后拉取最新镜像,使用最新镜像名

```
docker pull <latest-ci>
```

4\.删除旧的同名容器,避免运行新镜像失败

```
docker stop ci
docker rm ci
```

5\.运行镜像制造新容器ci且挂载宿主机测试用例文件夹,并且要绑定外部端口50051(ps:如果该端口被使用了，绑定就会失败)

```
docker run --name ci -d -p 50051:50051 -v <your_data_path>:/test/data <latest-ci>
```

如:
```
docker run --name ci -d -p 50051:50051 -v ./data:/test/data 10.249.12.83:5000/compilerhit/sysy-rv64-cpci:2.1
```

6\.在宿主机安装需要的python的grpc模块:

```
pip install grpcio grpcio-tools protobuf
```

### ci使用初始化(自动版:TODO,maybe not do)

1. 编辑配置文件(选项以及作用如下)

```
container_name:
registry_hub:
registry_account:
registry_password:
container_name:
```

1. 使用默认配置初始化ci使用(配置文件ci.config会生成在程序当前路径下)

```
ci init
```

1. 使用指定配置文件中的配置初始化ci,并且会在当前目录下生成配置文件，
   (如果配置文件路径错误，或者文件内容不合法，则使用默认配置)

```
ci init -config <config_path>
```

### ci使用流程

如果编译器项目进行了新的编译,可以使用-u标签指定先更新容器中的编译器再测试,(其中<test_folder1>等是指定的容器中/test/data下要测试的用例文件夹名):

```
./test -u <test_folder1> <test_folder2> ..
```

如果代码没有发生更新,要使用过去编译的compiler进行编译

```
./test <test_folder1> <test_folder2> ...
```

如果要测试容器中挂载到的 /test/下所有用例

```
./test all
```

### 更新

1. 增加执行时间获取  ():
每个测试样例测试后会在当行右侧显示标准程序执行时间和我们的程序执行时间
每个文件夹测试完成之后log目标程序执行时间总和在最下面