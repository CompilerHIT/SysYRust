### ci使用流程

1. 本地拉取docker image然后运行启动测试服务器的脚本(ps,注意版本)

登录局域网registry
docker login 10.249.12.83:5000
账号:root
密码:root

拉取镜像
docker pull riscv-ci:3.0

1. 运行镜像,挂载数据(测试数据应该放在data文件夹下面,详见https://github.com/cncsmonster/c-ci)

docker run -it -d -p 50051:50051 -v your_data_path:/test/data --name ci riscv-ci:3.0

1. 本地rpc调用镜像内的测试代码
   在根目录中,已经编译好了可执行程序test,
   test   ...
   将测试环境<your_data_path>目录下指定目录中的内容，并返回测试结果
   生成的自己的汇编代码myasm以及比对用的标准汇编代码tasm以及编译和运行过程中的提示文件log
   都会放到该路径下

比如你挂载了 /root/data 目录进容器的/test/data中
则在compiler2023的根目录下执行
test functional
则会测试 /root/data/functional文件夹中的所有用例