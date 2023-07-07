
c_path=$1
myasm_path=$2
out_path=$3
std_out_path="std_$out_path"
lib_path="./sylib.c"
myexe="mexe.out"
stdexe="stdexe.out"

if [ -z "$4" ]; then
    echo "第四个参数为空"

    gcc $myasm_path $lib_path -o $myexe
    start_time=$(date +%s%3N)
    ./"$myexe"  >$out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"my_time.txt"
    echo "" >>$out_path
    echo $? >>$out_path
    gcc $c_path $lib_path -O -o $stdexe
    start_time=$(date +%s%3N)
    ./"$stdexe"  > $std_out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"std_time.txt"
    echo "" >> $std_out_path
    echo $? >> $std_out_path
else
    echo "第四个参数为: $4"
    input_path=$4
    gcc $myasm_path $lib_path -o $myexe
    start_time=$(date +%s%3N)
    ./"$myexe"  <$input_path >$out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"my_time.txt"
    echo "" >>$out_path
    echo $? >>$out_path
    
    gcc $c_path $lib_path -O -o $stdexe
    start_time=$(date +%s%3N)
    ./"$stdexe"  <$input_path > $std_out_path
    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))
    echo $execution_time >>"std_time.txt"
    echo "" >> $std_out_path
    echo $? >> $std_out_path
fi
