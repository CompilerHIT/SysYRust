# 测试输入的若干个项目


sudo rm ./data/*log
sudo rm ./data/*.txt

./test -u
for arg in "$@"; do
    ./test -p  $arg
done
