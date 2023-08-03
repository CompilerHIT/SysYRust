for file in *.sy; do
	mv -- "$file" "${file%.sy}.c"
    gcc "$file" -S -o "${file}.s -O2"
done