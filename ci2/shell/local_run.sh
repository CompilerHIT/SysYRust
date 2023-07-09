# 调用本地的compiler编译程序 ,陆续执行目标路径内的用例并移出结果
# 从ci.cnf里面读取需要的路径,
# 参数
# 输入的路径,

# 启动远程测试的函数        (远程测试会自动监听)
function start_test(){
    # 第一个参数是挂载的文件夹地址
    mount_dir=$1
    info_path="$mount_dir/ci.info"
    cur_time=$(date)
    sudo echo "call at $cur_time" >> "$info_path"
}

# 获取本地路径,挂载路径
file_path="./ci2/ci.cnf"
# 获取本地挂载的路径
local_mount_dir=$(sed -n '1p' "$file_path")
back_dir=$(sed -n '2p' "$file_path")
time_limit=$(sed -n '3p' "$file_path")

# 首先compile->start_test,then call local_spy
# compile 传入的参数 1-2个  sy ,[in]
bash ./ci2/shell/compile.sh $1 $2
start_test
sudo python3 ./ci2/py/local_spy.py $local_mount_dir $back_dir $time_limit
