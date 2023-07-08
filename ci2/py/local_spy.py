import sys
import os
import subprocess
import time

# 本地监听服务器，监听请求，收到请求后往挂载的文件目录传入

# 
def call_listener(mount_dir:str,out_path:str,time_limit=20000):
    info_path=mount_dir+"/ci.info"
    initial_modification_time = os.path.getmtime(info_path)
    while True:
        # 获取当前的修改时间
        current_modification_time = os.path.getmtime(info_path)
        passed_time=current_modification_time-initial_modification_time
        if passed_time> time_limit:
            print("time out!")
            break
        if current_modification_time == initial_modification_time:
            time.sleep(0.05)
            continue
        subprocess.run(['bash','./mv_cmp.sh',mount_dir,out_path,time_limit])
        initial_modification_time = os.path.getmtime(info_path)
        break

# 三个参数,挂载文件夹,输出路径，时间限制
if __name__=="__main__":
    args=sys.argv[1:]
    # 挂载目录路径,
    mount_dir:str=args[0]
    # out文件夹路径
    out_path:str=args[1]
    time_limit=20000
    if len(args)>=3:
        time_limit=int(args[2],10)
    call_listener(mount_dir=mount_dir ,out_path=out_path,time_limit=time_limit)
    
    


