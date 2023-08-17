# IR 结构

代码存放于`src/ir/`路径下。
IR 为 SSA 形式，主要包括以下几个部分：

## 模块

输入文件在经过前端解析后会被整个为一个模块
整个模块的大致结构分为如下两个部分：

1. 全局变量和全局数组
2. 函数

```rust
pub struct Module {
    global_variable: Vec<(String, ObjPtr<Inst>)>,
    function: Vec<(String, ObjPtr<Function>)>,
}
```

## 函数

主要有以下几个部分构成：

```rust
pub struct Function {
    value: Value,
    return_type: IrType,
    parameters: HashMap<String, ObjPtr<Inst>>,
    index: Vec<ObjPtr<Inst>>,
    head_block: Option<ObjPtr<BasicBlock>>,
}
```

函数分为内部函数和外部函数。内部函数指在当前文件中定义的函数，其函数入口块部分为`Some(bb)`；而外部函数则是一些外部接口，包括标准库定义的函数（getint()等）和前端 ir 想要后端调用的一些函数（memset，自动并行化时使用的一些函数），其函数入口块部分为`None`。

## 基本块

主要结构如下：

```rust
pub struct BasicBlock {
    name: String,
    value: Value,
    inst_head: Inst,
    up_bb: Vec<ObjPtr<BasicBlock>>,
    next_bb: Vec<ObjPtr<BasicBlock>>,
}
```

函数中的基本块结构为图结构，而遍历整个图结构则通过访问`up_bb`和`next_bb`来实现。
基本块中保存指令的结构为双链循环链表，`inst_head`为表头，其随基本块创建时一起创建，是一条无意义的指令，只是为了便于实现指令的插入和删除而存在。

## 指令

主要结构如下：

```rust
pub struct Inst {
    user: User,
    list: IList<Inst>,
    kind: InstKind,
    /// 第一个bool为true时, 如果当前数组长度为0，则是未初始化的
    /// 第二个bool为true时，如果当前i32值为0，那么这个地方其实是被一个变量初始化的
    init: ((bool, Vec<(bool, i32)>), (bool, Vec<(bool, f32)>)),
    parent_bb: Option<ObjPtr<BasicBlock>>,
}
```

每一条指令都是一个新的变量，其值的类型主要为：

1. int：表示一个值为整型的变量
2. float：表示一个值为浮点型的变量
3. intptr：表示一个值为指向整型数的指针变量
4. floatptr：表示一个值为指向浮点型数的指针变量
5. void：表示一个变量，其内部没有值

`User`类用于管理指令的操作数和`use-define`关系。
`IList<Inst>`为侵入式链表，管理一个块中的指令的顺序关系。
`InstKind`管理指令类型。
`init`在当前指令为数组时有效，用于初始化数组。
`parent_bb`记录当前指令所属的块，为`None`时为全局变量。

# 优化

## 常规优化

- phi 优化：减少不必要的 phi 指令
- 死代码消除：删除不会被使用到的指令
- 数组和全局变量的归纳和转换：全局变量转换为局部变量，全局数组转换为局部数组，数组初始化识别
- 不可达路径删除

## 循环优化

循环优化主要顺序为：

1. 循环化简：将循环优化为简单形式，包括加入`preheader`,将多个`latch`合并为一个
2. 循环死代码消除：将循环中并不会被循环外使用且在循环内无意义的代码删除
3. 循环归纳和删除
4. 循环不变量外提
5. 循环展开
6. 自动并行化分析并修改 IR 结构以便于自动并行化
7. 归纳变量强度削减

### 循环归纳和删除

考虑如下代码

```c/c++
int main()
{
    int i = 0;
    int a[100] = {};
    int n = getint();
    int step = getint();
    int start = getint();
    int sum = 0;
    while (i < n) {
        a[i] = step;
        sum = sum + step;
        i = i + 1;
    }
    return sum;
}
```

循环归纳会对 sum 进行优化：

```c/c++
int main()
{
    int i = 0;
	int a[100] = {};
    int n = getint();
    int step = getint();
    int start = getint();
    int sum = start + n * step;
    while (i < n) {
        a[i] = step;
        i = i + 1;
    }
    return sum;
}
```

循环数组归纳会对 a[100]进行优化：

```c/c++
int main()
{
    int i = 0;
    int a[100] = {};
    int n = getint();
    int step = getint();
    int start = getint();
    int sum = start + n * step;
    memset(a+0, n, step);
    while (i < n) {
        i = i + 1;
    }
    return sum;
}
```

最后循环删除会将这个循环优化掉：

```c/c++
int main()
{
    int i = 0;
    int a[100] = {};
    int n = getint();
    int step = getint();
    int start = getint();
    int sum = start + n * step;
    memset(a+0, n, step);
    return sum;
}
```

### 自动并行化

自动并行化主要分析不同迭代次数在访问数组时是否会有`读-写`和`写-写`冲突。
分析的结果主要由依赖分析提供。
自动并行化与后端有如下接口：

```c
// 初始化线程池，在main函数中调用一次即可
void hitsz_thread_init();

// 创建一个新的线程，返回线程id
int hitsz_thread_create();

// 等待线程结束
void hitsz_thread_join();
```

自动并行化会将迭代空间切分成四份，考虑如下代码：

```c
int i = 0;
int n = getint();
int a[100];//假设已初始化
int b[100];//假设已初始化
int c[100];//假设已初始化
while (i < 0) {
	a[i] = b[i] * c[i];
	i = i + 1;
}
```

自动并行化后：

```c
int i = 0;
int n = getint();
int a[100];//假设已初始化
int b[100];//假设已初始化
int c[100];//假设已初始化
int n_thread = 0;
while (n_thread < 3) {
	if (hitsz_thread_creat == 0) {
		i = i + 1;
		n_thread = n_thread + 1;
	} else {
		break;
	}
}
while (i < 0) {
	a[i] = b[i] * c[i];
	i = i + 4;
}
hitsz_thread_join();
```

### 归纳变量强度削减

依赖与 SCEV 的分析，将形如

```c
int i = 0;
int b;
while (i < 0) {
	b = 4 * i;
	i = i + 4;
}
```

优化为：

```c
int i = 0;
int b = 0;
while (i < 0) {
	b = b + 4;
	i = i + 4;
}

```

## 函数内联

内联规则为将所有能内联的全都内联，即被调用函数为递归函数，或者被调用函数为函数自身，其他情况一律内联。

# 优化分析

## 函数调用图

将函数调用关系以图的形式呈现，在函数内联时使用

## 函数调用指令分析

分析当前函数是否是纯函数，在死代码消除时使用

## 支配树

计算数据流图的支配树，参考龙书的算法先计算支配关系，再构造出支配树。

## 循环识别

后序遍历支配树，收集到当前循环的所有 latch，然后从每个 latch 逆着数据流访问到循环头，并在这个过程中收集当前的循环块和子循环信息。循环结构主要如下：

```rust
pub struct LoopInfo {
    parent: Option<ObjPtr<LoopInfo>>,
    pre_header: Option<ObjPtr<BasicBlock>>,
    header: ObjPtr<BasicBlock>,
    latchs: Option<Vec<ObjPtr<BasicBlock>>>,
    exit_blocks: Option<Vec<ObjPtr<BasicBlock>>>,
    blocks: Vec<ObjPtr<BasicBlock>>,
    sub_loops: Vec<ObjPtr<LoopInfo>>,
}
```

- `parent`：当前循环的父循环，为最外层循环时为空
- `preheader`：循环识别时不存在，在循环化简后才会有
- `header`：循环头
- `latchs`：循环体内跳回循环头的块
- `exit_blocks`：循环体内跳出当前循环的块，不包括跳入子循环的块
- `blocks`：当前循环的块，不包括子循环的块
- `sub_loops`：子循环，可能有多个

## SCEV

主要结构：

```rust
pub struct SCEVExp {
    kind: SCEVExpKind,
    operands: Vec<ObjPtr<SCEVExp>>,
    scev_const: i32,
    bond_inst: Option<ObjPtr<Inst>>,
    in_loop: Option<ObjPtr<LoopInfo>>,
}

pub enum SCEVExpKind {
    SCEVConstant,
    SCEVUnknown,
    SCEVAddExpr,
    SCEVSubExpr,
    SCEVMulExpr,
    SCEVRecExpr,
    SCEVAddRecExpr,
    SCEVSubRecExpr,
    SCEVMulRecExpr,
}
```

根据 SCEVExpKind 来做简要介绍

### SCEVConstant

代表一个整型常量

### SCEVUnknown

本人实现的 SCEV 是以循环为单位，当指令不在循环中或者是一条不支持的指令，则会识别为当前类型，会绑定一条指令。

### SCEVAddExpr、SCEVSubExpr、SCEVMulExpr

为一条计算表达式，是两个 SCEVExp 计算的结果，其是为了表达某个结果而产生，不是从指令分析得到的，所以不绑定某条指令。

### SCEVRecExpr、SCEVAddRecExpr、SCEVSubRecExpr、SCEVMulRecExpr

通过分析某条指令得出的结果，即一条指令分析的结果如果不是 SCEVUnknow，就会是以上几种的一种。其中 SCEVRec 为识别 Phi 可能得到的结果，SCEVAddRecExpr 为识别 add 指令得到的结果，后面两个类似。
这些指令的操作数的含义 SCEVAddExpr 等的不同，后者表示的是两个表达式计算的结果，而前者的表达式的含义如下：

- 第一个操作数为 start，表示循环开始时的值
- 后续的操作数共同组成 step，表示每次循环迭代后递增的值
  考虑如下代码：

```c
int i = 0;
int b = 0;
while (i < 0) {
	b = i * 4;
	i = i + 4;
}
```

对应的 ir：

```llvm
define dso_local signext i32 @main() #0 {
bb_main:
  br label %bb_3

bb_3:
  %b = phi i32 [ 0, %bb_main ], [ %b_mul, %bb_1 ]
  %i = phi i32 [ 0, %bb_main ], [ %i_add, %bb_1 ]
  %cond = icmp slt i32 %i, 10
  br i1 %cond, label %bb_1, label %bb_2

bb_2:
  ret i32 %b

bb_1:
  %b_mul = mul i32 %i, 4
  %i_add = add i32 %i, 1
  br label %bb_3

}
```

其中：

```
i:SCEVRecExpr op[0,1]
b:SCEVREcExpr op[0,4]
cond:SCEVUnknow
b_mul:SCEVREcExpr op[0,4]
i_add:SCEVAddRecExpr op[1,1]
```

更复杂分析规则如下：
![CR Construction](https://private-user-images.githubusercontent.com/76612231/261257303-4011c061-8805-4815-a87a-979324554aba.png?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTEiLCJleHAiOjE2OTIyNjMzMzMsIm5iZiI6MTY5MjI2MzAzMywicGF0aCI6Ii83NjYxMjIzMS8yNjEyNTczMDMtNDAxMWMwNjEtODgwNS00ODE1LWE4N2EtOTc5MzI0NTU0YWJhLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFJV05KWUFYNENTVkVINTNBJTJGMjAyMzA4MTclMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjMwODE3VDA5MDM1M1omWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTA0MTFhNTMxNGFhOGU0ZmNlMGE3NzU4OGQ1MzFmYTY3MTAxMTY0MjhlNzgzY2IxYjU0MmM2NWVkNjAzMGM2YTUmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0JmFjdG9yX2lkPTAma2V5X2lkPTAmcmVwb19pZD0wIn0.IJ772sNnApk68tPr_w86OYMIwg4zhRx0ZZjTBDiKdLs)
参考：[(PDF) Symbolic Evaluation of Chains of Recurrences for Loop Optimization (researchgate.net)](https://www.researchgate.net/publication/2801813_Symbolic_Evaluation_of_Chains_of_Recurrences_for_Loop_Optimization)

## 依赖分析

TODO
