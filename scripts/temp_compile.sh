#!/bin/bash

source_folder="$1"
source_files=$(ls "${source_folder}" | grep '\.sy$')

compiler_path="./target/release/compiler"

rm -r compiler_product
# 检测compiler_product文件夹是否存在
if [ ! -d "compiler_product" ]; then
	mkdir compiler_product
fi

for file in ${source_files}; do
	echo "Compiling ${file}..."
	${compiler_path} "${source_folder}/${file}" -o "compiler_product/${file%.*}.s" -O1
	echo "Done!"
done
