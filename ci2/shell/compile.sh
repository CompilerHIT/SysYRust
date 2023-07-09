
# 本地编译目标文件

# 三个命令行参数:目标源代码路径,目标输入路径,
# 判断路径名是函数名还是文件夹名

get_basename(){
    local filepath="$1"
    # 使用basename命令获取文件名（不包括路径）
    local filename=$(basename "$filepath")
    # 使用参数扩展去掉后缀
    local filename_without_extension="${filename%.*}"
    # 输出去掉后缀的文件名
    echo "$filename_without_extension"
}

# 获取输出
get_output() {
  local filepath="$1"
  # 使用basename命令获取文件名（不包括路径）
  local filename=$(basename "$filepath")
  # 使用参数扩展去掉后缀
  local filename_without_extension="${filename%.*}"
  # 输出去掉后缀的文件名
  echo "$filename_without_extension.out"
}

# 启动远程测试的函数  (远程测试会自动监听结果)
function start_test(){
    # 第一个参数是挂载的文件夹地址
    mount_dir=$1
    info_path="$1/ci.info"
    cur_time=$(date)
    echo "call at $cur_time" >> "$info_path"
}

# 判断路径名是函数名还是文件夹名
function check_path_type() {
    path="$1"
    if [ -d "$path" ]; then
        echo "1"        #文件夹名字
    else 
        echo "2"    #文件名
    fi
}

# 获取本地路径,挂载路径
file_path="./ci2/ci.cnf"
# 获取本地挂载的路径
local_path=$(sed -n '1p' "$file_path")


# 两个命令行参数
# 1. 目标sy程序路径
# 2. (可选)目标程序执行时输入(默认为没有)
sy_path=$1
if [ $# -ge 2 ]; then
  echo "目标程序有输入"
  input_path=$3
  basename=$(get_basename $input_path)
  sudo cp "$input_path" "$local_path/$basename.in"
  sudo cp "$sy_path" "$local_path/$basename.sy"
  asm_path="$local_path/$basename.s"
  sudo ./target/release/compiler "$sy_path" -S -o "$asm_path" -O1
else
  echo "目标程序无输入"
  basename=$(get_basename $input_path)
  sudo cp "$sy_path" "$local_path/$basename.sy"
  asm_path="$local_path/$basename.s"
  sudo ./target/release/compiler "$sy_path" -S -o "$asm_path" -O1
fi