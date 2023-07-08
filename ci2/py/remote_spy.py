import os
import time
import subprocess

def get_all_file_paths(directory):
    file_paths = []
    for root, dirs, files in os.walk(directory):
        for file in files:
            file_path = os.path.join(root, file)
            file_paths.append(file_path)
    return file_paths

# 获取所有测试单元,并按照字典顺序进行排序
def get_all_test_paths(path:str):
    sysets=set()
    insets=set()
    asms=set()
    files=get_all_file_paths(path)
    for file in files :
        # 去掉后缀名,判断是否一致
        basename, extension = os.path.splitext(file)
        if extension== ".sy":
            sysets.add(basename)
        elif extension==".in":
            insets.add(basename)
        elif extension==".s":
            asms.add(basename)
    basename_list=[]
    name_map=dict()
    for basename in sysets:
        testunit:Unit=Unit(sy=basename+".sy")
        testunits.append(testunit)
        basename_list.append(basename)
        name_map[basename]=testunit
        if basename in insets :
            testunit.input=basename+".in"
        if basename in asms:
            testunit.asm=basename+".s"
    # 对testunits进行排序,按照首个字母下标进行升序排序
    basename_list=sorted(basename_list)
    final_testunits=[]
    for basename in basename_list :
        final_testunits.append(name_map[basename])
    testunits=final_testunits
    return testunits
# 测试单元
class Unit:
    # 默认超时时间2000毫秒
    def __init__(self,sy,asm,input=""):
        self.input=input
        self.sy=sy
        self.asm=asm
   

def check_file_modification(filename):
    # 获取文件的初始修改时间
    initial_modification_time = os.path.getmtime(filename)
    print(initial_modification_time)
    while True:
        # 获取当前的修改时间
        current_modification_time = os.path.getmtime(filename)
        if current_modification_time == initial_modification_time:
            time.sleep(0.2)
            continue
        print("remote run")
        unit:Unit=get_all_test_paths()[0]
        if unit.input=="":
            subprocess.run(['bash','./remote_run.sh',unit.sy],check=True)
        else:
            subprocess.run(['bash','./remote_run.sh',unit.sy,unit.input],check=True)
        initial_modification_time = os.path.getmtime(filename)
        # 更新初始修改时间

if __name__ == "__main__":
    filename = "./ci.info"
    check_file_modification(filename)