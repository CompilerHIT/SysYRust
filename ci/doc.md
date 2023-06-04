### ci使用前置(手动版)

1\.docker 本地registry部署在局域网中，未配置https,所以需要切换使用http client来允许docker进行不安全的pull

```
sudo touch /etc/docker/daemon.json

sudo echo '{ "insecure-registries":["10.249.12.83:5000"] }' > /etc/docker/daemon.json

sudo systemctl restart docker
```

2\.然后登录该局域docker registry

```
docker login docker login 10.249.12.83:5000 -u root -p root
```

3\.然后拉取最新镜像,为最新的测试镜像名

```
docker pull <latest-ci>
```

当前最新名为riscv-ci:3.0

4\.删除旧的同名容器,避免运行新镜像失败

```
docker stop ci
docker rm ci
```

5\.运行镜像制造新容器且挂载宿主机测试用例文件夹

```
docker run --name ci -v <your_data_path>:/test/data <latest-ci>
```

### ci使用初始化(自动版:TODO)

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
test all
```