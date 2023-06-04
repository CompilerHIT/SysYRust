import ci_pb2;
import ci_pb2_grpc;
import grpc

import grpc
import sys
import subprocess

channel = grpc.insecure_channel('localhost:50051')
stub = ci_pb2_grpc.GreeterStub(channel)


# 每次测试之前，会把可执行程序复制到ci之中


# 获取命令行参数
# 对于每个命令行参数,发起一次测试
for path in sys.argv[1:] :
    # print("get",path)
    response=stub.CallTest(ci_pb2.TestRequest(path=path))
    print(response.retMsg)