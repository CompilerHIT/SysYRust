sudo rm ./data/*log
sudo rm ./data/*.txt
rm ./data/mout
rm ./data/tout
rm ./data/mexe
rm ./data/texe

./test -u
for arg in "$@"; do
    ./test -p  $arg
done
