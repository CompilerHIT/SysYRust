# 从挂载路径中移动出并且比较输出结果
# 只传入一个参数,用例的basename
# 比如用例文件a.sy,则basename为a
# local run

# 读取文件的前四行并存储到Shell变量中
local_path=$1
back_dir=$2

# # 输出变量的值
# echo "local_path: $local_path"
# echo "back_dir: $back_dir"
# TODO,获取输入
out_path="$local_path/out_$basename.txt"
std_out_path="$local_path/std_out_$basename.txt"
my_time_path="$local_path/my_time.txt"
std_time_path="$local_path/std_time.txt"
log_path="$local_path/log"

# 移动到back_dir中
mv "$out_path" "$back_dir/"
mv "$std_out_path" "$back_dir/"
cat "$my_time_path" >> "$back_dir/my_time.txt"
cat "$std_time_path" >> "$back_dir/std_time.txt"
cat "$log_path" >> "$back_dir/log"

# 删除 挂载文件家中内容
rm "$my_time_path"
rm "$std_time_path"
rm "$out_path"
rm "$std_out_path"
rm "$log_path"

