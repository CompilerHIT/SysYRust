#!/bin/bash

source_folder="$1"

source_files=$(ls "${source_folder}" | grep '\.sy$')

llc_target="-march=riscv64"

for file in $source_files; do
	echo "Processing file: ${file}"
	./target/debug/compiler "${source_folder}${file}" -O1

	if [ $? -eq 0 ]; then
		echo "Code generation successful!!!"
	else
		echo "Code generatiton failed!!!"
		break
	fi

	llc "${llc_target}" -opaque-pointers -o "testcase.s" "dump_opt.ll"
	# 检查编译结果
	if [ $? -eq 0 ]; then
		echo "Compilation successful for ${file}"
	else
		echo "Compilation failed for ${file}"
		read -p "Press 'n' to continue or any other key to exit: " choice
		if [ "$choice" = "n" ]; then
			echo
		else
			exit 1
		fi
	fi

	echo "---------------------------------------"
	echo ""
done
