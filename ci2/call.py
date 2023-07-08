import os
import subprocess
import sys

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
    outsets=set()
    files=get_all_file_paths(path)
    for file in files :
        # 去掉后缀名,判断是否一致
        basename, extension = os.path.splitext(file)
        if extension== ".sy":
            sysets.add(basename)
        elif extension==".in":
            insets.add(basename)
        elif extension==".out":
            outsets.add(basename)
    basename_list=[]
    name_map=dict()
    for basename in sysets:
        testunit=Unit(sy=basename+".sy")
        testunits.append(testunit)
        basename_list.append(basename)
        name_map[basename]=testunit
        if basename in insets :
            testunit.input=basename+".in"
    # 对testunits进行排序,按照首个字母下标进行升序排序
    basename_list=sorted(basename_list)
    final_testunits=[]
    for basename in basename_list :
        final_testunits.append(name_map[basename])
    testunits=final_testunits
    return testunits
    
    
# 测试文件夹
def test_dir(path:str):
    units=get_all_test_paths(path=path)
    # print(2)
    for unit in units:
        # print(1)
        unit:Unit=unit
        unit.test()

# 测试单元
class Unit:
    # 默认超时时间2000毫秒
    def __init__(self,sy,input="",timeout=2000):
        self.input=input
        self.sy=sy
        self.timeout=timeout
        
    #测试该测试单元,执行前应该先指定使用的编译器
    def test(self):
        # 运行脚本进行测试,local_run
        if self.input=="":
            subprocess.run(['bash','./ci2/shell/local_run.sh',self.sy])
        else:
            subprocess.run(['bash','./ci2/shell/local_run.sh',self.sy,self.input])
   

if __name__=='__main__':
    # 传入命令行参数,文件夹名或者文件名
    argv=sys.argv[1:]
    print("test"+str(argv))
    for path in argv:
        if os.path.isdir(path):
            test_dir(path=path)
        else:
            basename, extension = os.path.splitext(path)
            input=basename+".in"
            if os.path.isfile(input):
                unit:Unit=Unit(input=input,sy=path)
                unit.test()
            else:
                unit:Unit=Unit(sy=path)
                unit.test()
    print("finish")       