# 判断路径名是函数名还是文件夹名
function check_path_type() {
    path="$1"
    if [ -d "$path" ]; then
        echo "1"        #文件夹名字
    else 
        echo "2"    #文件名
    fi
}
