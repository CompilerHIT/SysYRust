
cp ./target/release/compiler ./data/
if [ $? -eq 0 ] ; then 
    echo "succes update compiler"
else 
    exit -1
fi
sudo rm ./data/*log
sudo rm ./data/*.txt
rm ./data/mout
rm ./data/tout
rm ./data/mexe
rm ./data/texe

for arg in "$@"; do
    ./test -p  $arg
done
