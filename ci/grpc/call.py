import ci_pb2;
import ci_pb2_grpc;
import grpc

import grpc
import sys

channel = grpc.insecure_channel('localhost:50051')
stub = ci_pb2_grpc.GreeterStub(channel)

# 获取命令行参数
# 对于每个命令行参数,发起一次测试
for path in sys.argv[1:] :
    # print("get",path)
    response=stub.CallTest(ci_pb2.TestRequest(path=path))
    print(response.retMsg)