针对用例integer-division
因为之前使用的handlespill的策略问题，产生了很多额外的load store,
所以消除后性能提升很大。

原本汇编20000+(开启了其他优化),开启该优化后代码量剩余17000+