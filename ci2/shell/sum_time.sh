# 统计my_time.txt和std_time.txt文件中的所有内容到文件time.txt中

#!/bin/bash

file_path="./ci2/ci.cnf"
# 读取文件的前四行并存储到Shell变量中
local_path=$(sed -n '1p' "$file_path")


input_file="$back_dir/my_time.txt"    # 输入文件名
input_file2="$back_dir/std_time.txt"    # 输入文件名

output_file="time.txt"  # 输出文件名

total=0  # 累加总和
total2=0    #累加标准时间

# 逐行读取输入文件并进行累加
while IFS= read -r line; do
  # 将每行的数字累加到总和中
  total=$((total + line))
done < "$input_file"

while IFS= read -r line; do
  # 将每行的数字累加到总和中
  total2=$((total2 + line))
done < "$input_file2"


# 进行除法计算，保留小数点后两位
result=$(echo "scale=2; $dividend / $divisor" | bc)

# 将结果转换为百分比形式，保留两位小数
percentage=$(echo "scale=2; $result * 100" | bc)

# 将累加总和写入输出文件
echo "my/std:$total/$total2" > "$output_file"
echo "百分比形式: $percentage%"
