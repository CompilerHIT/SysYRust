sudo rm ./data/*log
sudo rm ./data/*.txt

./test -u
for arg in "$@"; do
    ./test -p  $arg
done
