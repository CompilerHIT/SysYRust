#!/bin/bash

set -e

if [ ! -d "png" ]; then
	mkdir png
fi

cp dump.ll png
cp dump_opt.ll png
cd png

opt -dot-cfg dump.ll -disable-output -enable-new-pm=0 -opaque-pointers

for file in .*.dot; do
	dot -Tpng "$file" -o "${file%.dot}.png"
done

rm .*.dot

opt -dot-cfg dump_opt.ll -disable-output -enable-new-pm=0 -opaque-pointers

for file in .*.dot; do
	dot -Tpng "$file" -o "${file%.dot}_opt.png"
done

rm .*.dot
