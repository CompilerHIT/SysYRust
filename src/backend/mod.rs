mod asm_builder;
pub mod block;
pub mod func;
mod generate;
pub mod instrs;
pub mod module;
pub mod operand;
pub mod opt;
pub mod regalloc;
pub mod simulator;
pub mod structs;

pub mod parrallel;
use std::fs::File;
use std::io::Write;

use crate::backend::module::AsmModule;
use crate::backend::opt::BackendPass;
use crate::config;
use crate::utility::ObjPool;

use self::func::Func;
use self::instrs::{Context, LIRInst, ObjPtr, BB};

pub struct BackendPool {
    func_pool: ObjPool<Func>,
    block_pool: ObjPool<BB>,
    inst_pool: ObjPool<LIRInst>,
    context_pool: ObjPool<Context>,
}

impl BackendPool {
    pub fn new() -> Self {
        Self {
            func_pool: ObjPool::new(),
            block_pool: ObjPool::new(),
            inst_pool: ObjPool::new(),
            context_pool: ObjPool::new(),
        }
    }

    pub fn put_func(&mut self, func: Func) -> ObjPtr<Func> {
        self.func_pool.put(func)
    }

    pub fn put_block(&mut self, block: BB) -> ObjPtr<BB> {
        self.block_pool.put(block)
    }

    pub fn put_inst(&mut self, inst: LIRInst) -> ObjPtr<LIRInst> {
        self.inst_pool.put(inst)
    }

    pub fn put_context(&mut self, context: Context) -> ObjPtr<Context> {
        self.context_pool.put(context)
    }

    pub fn free_all(&mut self) {
        self.func_pool.free_all();
        self.block_pool.free_all();
        self.inst_pool.free_all();
        self.context_pool.free_all();
    }
}

pub fn generate_asm(
    in_path: &str,
    path: &str,
    row_path: &str,
    module: &mut AsmModule,
    is_opt: bool,
) {
    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(e) => panic!("Create    output path error: {}", e),
    };
    writeln!(file, "	.file	\"{}\"", in_path).unwrap();
    writeln!(file, "	.option pic").unwrap();
    writeln!(file, "    .text").unwrap();
    let mut pool = BackendPool::new();
    let mut file2 = File::create(row_path).unwrap();

    //构造
    module.build_v4(&mut file, &mut file2, &mut pool, is_opt);
    // module.generate_row_asm(&mut file2);

    // 后端优化
    if is_opt {
        BackendPass::new(ObjPtr::new(module)).run_pass(&mut pool);
    }
    // 检查地址溢出，插入间接寻址
    module.handle_overflow(&mut pool);

    if is_opt {
        //最后进行一次寄存器分配与合并
        config::record_event("start merge reg");
        module.final_realloc(&mut pool);
        config::record_event("finish merge reg");

        // 再次进行指令重排
        // module.re_list_scheduling();

        // 额外的块优化处理
        BackendPass::new(ObjPtr::new(module)).run_addition_block_pass(&mut pool);
    }

    //生成抽象汇编
    // module.generate_row_asm(&mut file2);
    let thread = false;
    // let thread = is_opt;
    if thread {
        writeln!(
            file,
            "
        .text
        .align	1
        .globl	hitsz_thread_init
        .type	hitsz_thread_init, @function
    hitsz_thread_init:
        addi	sp,sp,-16
        sd	ra,8(sp)
        call	pthread_self@plt
        lla	a3,.LANCHOR0
        lw	a5,0(a3)
        ld	ra,8(sp)
        slli	a4,a5,3
        add	a4,a3,a4
        addiw	a5,a5,1
        sw	a5,0(a3)
        sd	a0,8(a4)
        addi	sp,sp,16
        jr	ra
        .size	hitsz_thread_init, .-hitsz_thread_init
        .align	1
        .globl	get_next_pc
        .type	get_next_pc, @function
    # call完这个函数之后a0中的值即为call指令后下一条指令的地址
    get_next_pc:
        mv a0,ra
        ret
        .align	1
        .globl	hitsz_thread_self
        .type	hitsz_thread_self, @function
    hitsz_thread_self:
        addi	sp,sp,-32
        sd	ra,24(sp)
        sd	s0,16(sp)
        addi	s0,sp,32
        call	pthread_self@plt
        mv	a5,a0
        sd	a5,-32(s0)
    .L6:
        sw	zero,-20(s0)
        j	.L2
    .L5:
        lla	a4,tids
        lw	a5,-20(s0)
        slli	a5,a5,3
        add	a5,a4,a5
        ld	a5,0(a5)
        ld	a4,-32(s0)
        bne	a4,a5,.L3
        lw	a5,-20(s0)
        j	.L7
    .L3:
        lw	a5,-20(s0)
        addiw	a5,a5,1
        sw	a5,-20(s0)
    .L2:
        lla	a5,num_tid
        lw	a4,0(a5)
        lw	a5,-20(s0)
        sext.w	a5,a5
        blt	a5,a4,.L5
        j	.L6
    .L7:
        mv	a0,a5
        ld	ra,24(sp)
        ld	s0,16(sp)
        addi	sp,sp,32
        jr	ra
        .size	hitsz_thread_self, .-hitsz_thread_self
        .align	1
        .globl	hitsz_thread_join
        .type	hitsz_thread_join, @function
    hitsz_thread_join:
        addi	sp,sp,-32
        sd	ra,24(sp)
        sd	s0,16(sp)
        addi	s0,sp,32
        call	pthread_self@plt
        mv	a5,a0
        mv	a4,a5
        lla	a5,tids
        ld	a5,0(a5)
        bne	a4,a5,.L9
        li	a5,1
        sw	a5,-20(s0)
        j	.L10
    .L11:
        lla	a4,tids
        lw	a5,-20(s0)
        slli	a5,a5,3
        add	a5,a4,a5
        ld	a5,0(a5)
        li	a1,0
        mv	a0,a5
        call	pthread_join@plt
        lw	a5,-20(s0)
        addiw	a5,a5,1
        sw	a5,-20(s0)
    .L10:
        lla	a5,num_tid
        lw	a4,0(a5)
        lw	a5,-20(s0)
        sext.w	a5,a5
        blt	a5,a4,.L11
        lla	a5,num_tid
        li	a4,1
        sw	a4,0(a5)
        j	.L13
    .L9:
        li	a0,0
        call	pthread_exit@plt
    .L13:
        nop
        ld	ra,24(sp)
        ld	s0,16(sp)
        addi	sp,sp,32
        jr	ra
        .size	hitsz_thread_join, .-hitsz_thread_join
        .align	1
        .globl	hitsz_thread_create
        .type	hitsz_thread_create, @function
    # 创建线程
    hitsz_thread_create:
        addi sp,sp,-32
        sd ra,8(sp)
        sd s0,0(sp)
        sd	s1,16(sp)
        lla a0,tmp_mem
        sd	ra,0(a0)
        sd	s0,8(a0)
        sd s1,16(a0)
        lla s0,tids
        lla s1,num_tid
        # 首先判断当前是否是main线程
        call pthread_self@plt
        ld	s1,0(s0)
        beq a0,s1,LBB_open_thread
    LBB_leave_create_thread:
        mv	a0,zero
        ld	ra,8(sp)
        ld s0,0(sp)
        ld	s1,16(sp)
        addi sp,sp,32
        jr ra
    LBB_open_thread:
        call get_next_pc
        addi a2,a0,20
        add	a0,sp,24
        mv	a1,zero
        mv	a3,zero
        call pthread_create@plt
        # 恢复该线程应该有的sp,该线程的sp应该存在对应
        j	LBB_main_leave
        j	LBB_sun_leave
    LBB_main_leave:
        # 创建子线程后主线程的行为
        lla s0,tids
        lla s1,num_tid
        ld	a0,24(sp)	#获取创建的子线程的id
        lw	a1,0(s1)	#获取当前线程数量
        mv	a2,a1		#暂存原本线程数到a2
        slli	a1,a1,3	#获取线程数组偏移
        add s0,s0,a1	#获取当前线程编号数组要存入的位置
        sd	a0,0(s0)	#存入数组
        addi a0,a2,1	#线程数+1
        sd	a0,0(s1)	#新线程数存入num_tid
        # 返回新线程数对应编号,也是原本线程
        ld	ra,8(sp)
        ld s0,0(sp)
        ld	s1,16(sp)
        mv	a0,zero	#a0返回主线程id
        addi sp,sp,32
        jr ra
    LBB_sun_leave:
        # 创建子线程后子线程的行为
        # 首先从对应栈区域恢复之前的ra(之前的ra是调用者ra)
        # 从该位置恢复调用者ra,并且把数据保存到调用者ra上
        call hitsz_thread_self
        la	a1,tmp_mem
        ld	ra,0(a1)
        ld	s0,8(a1)
        ld	s1,16(a1)
        jr	ra
        .size	hitsz_thread_create, .-hitsz_thread_create
        
        
        .globl	num_tid
        .globl	tids
        .bss
        .align	3
        .set	.LANCHOR0,. + 0
        .type	num_tid, @object
        .size	num_tid, 4
    num_tid:
        .zero	4
        .zero	4
        .type	tids, @object
        .size	tids, 800
    tids:
        .zero	800
    # 用来进行缓存的空间
        .globl	tmp_mem
        .bss
        .align	3
        .type	tmp_mem, @object
        .size	tmp_mem, 800
    tmp_mem:
        .zero	800
        "
        )
        .unwrap();
    }

    //生成汇编
    module.generate_asm(&mut file, &mut pool);

    //释放
    pool.free_all();

    // writeln!(file, "    .ident	\"GCC: (Ubuntu 9.4.0-1ubuntu1~20.04) 9.4.0\"");
    writeln!(file, "    .section	.note.GNU-stack,\"\",@progbits").unwrap();
}
