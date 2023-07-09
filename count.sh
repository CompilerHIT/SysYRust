#!/bin/bash

# 指定要统计行数的文件夹路径
folder_path="$1"

# 检查命令行参数是否为空
if [ -z "$folder_path" ]; then
  echo "Please provide a folder path as a command line argument."
  exit 1
fi

# 使用 find 命令查找指定文件夹下的所有文件，并计算行数
total_lines=0
while read -r lines filename; do
  ((total_lines += lines))
  echo "$filename: $lines"
done < <(find "$folder_path" -type f -exec wc -l {} +)

# 输出总行数
echo "Total lines: $total_lines"
