#!/bin/bash

source_folder="./data/functional/"

source_files=$(ls "${source_folder}" | grep '^[0-9].*\.sy$')

set -e

llc_target="-march=riscv64"

for file in $source_files; do
	echo "Processing file: ${file}"
	./target/debug/compiler "${source_folder}${file}"
	llc "${llc_target}" -opaque-pointers -o "testcase.s" "dump.ll"
	# 检查编译结果
	if [ $? -eq 0 ]; then
		echo "Compilation successful for ${file}"
	else
		echo "Compilation failed for ${file}"
	fi

	echo "---------------------------------------"
	echo ""
done
