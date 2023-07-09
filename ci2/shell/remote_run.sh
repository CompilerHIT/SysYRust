cur_time=$(date)


sy_path=$1
myasm_path=$2


get_basename(){
    local filepath="$1"
    # 使用basename命令获取文件名（不包括路径）
    local filename=$(basename "$filepath")
    # 使用参数扩展去掉后缀
    local filename_without_extension="${filename%.*}"
    # 输出去掉后缀的文件名
    echo "$filename_without_extension"
}

basename=$(get_basename "$sy_path")
out_path="./out_$basename.txt"
std_out_path="./std_out_$basename.txt"
lib_path="./sylib.c"
myexe="mexe.out"
stdexe="stdexe.out"
compile_log="./log"

rm "$myexe"
rm "$stdexe"

if [ -z "$3" ]; then
    echo "无输入"
    gcc $myasm_path $lib_path -o $myexe >>"$compile_log"
    start_time=$(date +%s%3N)
    ./"$myexe"  >$out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"my_time.txt" 
    echo "" >>$out_path
    echo $? >>$out_path
    gcc $sy_path $lib_path -O -o $stdexe
    start_time=$(date +%s%3N)
    ./"$stdexe"  > $std_out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"std_time.txt"
    echo "" >> $std_out_path
    echo $? >> $std_out_path
else
    echo "有输入: $3"
    input_path=$3
    gcc $myasm_path $lib_path -o $myexe >>"$compile_log"
    start_time=$(date +%s%3N)
    ./"$myexe"  <$input_path >$out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"my_time.txt"
    echo "" >>$out_path
    echo $? >>$out_path
    
    gcc $sy_path $lib_path -O -o $stdexe
    start_time=$(date +%s%3N)
    ./"$stdexe"  <$input_path > $std_out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"std_time.txt"
    echo "" >> $std_out_path
    echo $? >> $std_out_path
fi

# rm "*.sy"
# rm "*.in"


echo "remote react at $cur_time" >> "./ci.info"

