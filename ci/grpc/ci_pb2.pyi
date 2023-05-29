from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class TestReply(_message.Message):
    __slots__ = ["retMsg"]
    RETMSG_FIELD_NUMBER: _ClassVar[int]
    retMsg: str
    def __init__(self, retMsg: _Optional[str] = ...) -> None: ...

class TestRequest(_message.Message):
    __slots__ = ["path"]
    PATH_FIELD_NUMBER: _ClassVar[int]
    path: str
    def __init__(self, path: _Optional[str] = ...) -> None: ...
