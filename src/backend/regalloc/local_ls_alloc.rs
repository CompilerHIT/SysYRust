// 进行了块局部优化的线性扫描寄存器分配
// 对整个函数寄存器分配的问题，取每个块的寄存器分配问题为子问题
// 使用子问题的解来合成整个寄存器分配问题的解

pub struct Allocator {}
