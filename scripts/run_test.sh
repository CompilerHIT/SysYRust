#!/bin/bash

if [[ $# -gt 1 ]]; then
	source_folder="$2"
	if [ "$1" == "-c" ]; then
		continue_flag='2'
	else
		echo "Usage: run_test.sh [-c] [source_folder]"
	fi
else
	source_folder="$1"
	continue_flag='1'
fi

# 测试文件总数，初始化为0，每测试一个文件，加1
total_file_num=0
# 通过的测试文件数，初始化为0，每通过一个文件，加1
pass_num=0
# 未通过的测试文件数，初始化为0，每未通过一个文件，加1
fail_num=0

source_files=$(ls "${source_folder}" | grep '\.s$')

for file in ${source_files}; do
	echo "Testing ${file}..."

	# 使用gcc将其与./lib/libsysy.a链接
	test_file="${file%.*}exe.out"
	gcc "${source_folder}/${file}" -L./lib -lsysy -static -o "${test_file}"

	# 查看./performance文件夹下是否有对应的.in文件作为输入
	out_file="${file%.*}.out"
	if [ -f "./performance/${file%.*}.in" ]; then
		# 如果有，则将其作为可执行文件的输入,并将运行结果输出到对应的.out文件
		./"${test_file}" <"./performance/${file%.*}.in" >"${out_file}"
	else
		# 如果没有，则将其作为输入
		./"${test_file}" >"${out_file}"
	fi

	# 将程序的返回值也保存在.out文件中
	echo $? >>"${out_file}"

	# 测试文件总数加1
	total_file_num=$(echo "${total_file_num} + 1" | bc)

	# 逐行比对.out文件和./performance文件夹下的.out文件
	diff_output=$(diff -y --suppress-common-lines "${out_file}" "./performance/${out_file}")
	# 如果两个文件不一致，则输出错误的行数和相应的错误信息
	if [ $? -ne 0 ]; then
		echo "Error in ${file}!"
		echo "${diff_output}"

		# 未通过的测试文件数加1
		fail_num=$(echo "${fail_num} + 1" | bc)

		# 如果不是继续运行模式，则询问是否继续运行
		if [ ${continue_flag} -ne '2' ]; then
			# 询问是否继续运行
			read -p "Continue? (y/n) " continue_flag
			if [ "${continue_flag}" != 'y' ]; then
				break
			fi
		fi
		echo "---------------"
	else
		# 通过的测试文件数加1
		pass_num=$(echo "${pass_num} + 1" | bc)

		echo "Passed ${file}!"
		echo "---------------"
	fi
	# 删除掉临时生成的.out文件和可执行文件
	rm "${out_file}" "${test_file}"
done

# 所有测试结束，打印结果和统计信息
echo ""
echo "Test finished!"
echo "Total: ${total_file_num} files, ${pass_num} passed, ${fail_num} failed."
