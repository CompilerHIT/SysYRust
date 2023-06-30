docker stop ci
docker rm ci
docker login 10.249.12.83:5000
docker pull 10.249.12.83:5000/compilerhit/sysy-rv64-cpci:3.4
docker run --name ci -d -p 50051:50051 -v ./data:/test/data 10.249.12.83:5000/compilerhit/sysy-rv64-cpci:3.4
