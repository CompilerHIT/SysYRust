# 记录开始时间
start_time=$(date +%s%3N)

# 执行程序
./a.out

# 记录结束时间
end_time=$(date +%s%3N)

# 计算执行时间
execution_time=$((end_time - start_time))