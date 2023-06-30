cargo build
sudo rm ./ci/data/log
sudo rm ./ci/data/fail.log
./test -u -p hidden_functional

