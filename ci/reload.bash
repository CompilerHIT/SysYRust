docker stop ci
docker rm ci
docker login 10.249.12.83:5000
docker pull 10.249.12.83:5000/compilerhit/sysy-rv64-cpci:5.0
data_path=$1
echo "share data path:${data_path}"
docker run --name ci -d -p 50051:50051 -v ${data_path}:/test/data 10.249.12.83:5000/compilerhit/sysy-rv64-cpci:5.0
