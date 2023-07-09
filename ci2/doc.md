# 实际开发板上的ci

### 工作流设计

1. 挂载我们的一个目录到板子上指定位置，并且启动使用机上的监控程序监控开发板上挂载的文件夹中的ci.info文件

   使用机和开发板通过这个共享的目录进行交流

2. 板子上共享的文件夹中运行一个实时的监控程序spy.py，监控ci.info

3. 当监控到ci.info这个文件发生改变的时候，就会重新地执行一遍run.sh脚本，执行完成之后便会修改ci.info文件。 当检测到发生修改,则宿主机上就会执行local_run.sh脚本,把需要转移的输出,比如 时间,编译中间产物, 目标程序执行结果，标准目标程序执行结果转移出来，然后在使用机上执行对目标程序(从我们编译出的汇编得到的可执行程序)的执行解雇哦与标准目标程序(标准编译器直接编译源代码得到的结果) 进行对比,
   然后把对比结果输入到挂载目录外的当前目录下的一个文件中

### ci部署

1. 开发板上，登录后:

   ```
   mkdir ~/remote_mount_dir
   ```

2. 主机上,具例如下:

   ```
   sudo mkdir ./ci2/local_mount_dir
   sudo mkdir ./ci2/out
   # 如果该目录已经挂载过了,还需要刷新下 
   #fusermount -u ./ci2/local_mount_dir
   #然后挂载需要挂载的中转目录
   sudo bash ./ci2/shell/mount.sh yjh@10.249.10.164 ./ci2/local_mount_dir /home/yjh/remote_mount_dir
   # 然后复制需要的文件进入中转目录
   cp ./ci2/lib/sylib* ./ci2/local_mount_dir/
   cp ./ci2/shell/remote_run.sh ./ci2/local_mount_dir/
   cp ./ci2/py/remote_spy.py  ./ci2/local_mount_dir/ 
   ```

3. 把参数输入到./ci2/ci.cnf内部,从上到下依次为 中转挂载文件夹,回放文件夹,时间限制
   ```
   ./ci2/local_mount_dir
   ./ci2/out
   100000
   ```
4. 开发板上,运行启动
   在开发板中设置挂载的文件夹为全可用(避免因为权限问题执行中断),然后在开发板上执行:
   ```
   cd ~/remote_mount_dir
   touch ./ci.info
   python3 remote_spy.py
   ```
   来启动开发板部分程序

### ci的使用

call.py中输入文件
```
python3 ./ci2/call.py <file_path/dir>
```