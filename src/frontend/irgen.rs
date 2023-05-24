use super::context::Type;
use super::{ast::*, context::Context};
use crate::frontend::error::Error;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
use crate::utility::{ObjPool, ObjPtr};

struct Kit {
    context_mut: &'static mut Context,
    pool_inst_mut: &'static mut ObjPool<Inst>,
    pool_func_mut: &'static mut ObjPool<Function>,
    pool_bb_mut: &'static mut ObjPool<BasicBlock>,
}

impl Kit {
    pub fn push_inst(&mut self, inst_ptr: ObjPtr<Inst>) {
        self.context_mut.push_inst_bb(inst_ptr);
    }

    pub fn add_var(&mut self, s: &str, tp: Type, is_array: bool, dimension: Vec<i64>) {
        self.context_mut.add_var(s, tp, is_array, dimension);
    }

    pub fn update_var(&mut self, s: &str, inst: ObjPtr<Inst>) -> bool {
        self.context_mut.update_var_scope(s, inst)
    }
}

pub fn irgen(
    compunit: &'static mut CompUnit,
    module_mut: &'static mut Module,
    pool_inst_mut: &'static mut ObjPool<Inst>,
    pool_bb_mut: &'static mut ObjPool<BasicBlock>,
    pool_func_mut: &'static mut ObjPool<Function>,
) {
    let mut pool_scope = ObjPool::new();
    let context_mut = pool_scope.put(Context::make_context(module_mut)).as_mut();
    let mut kit_mut = Kit {
        context_mut,
        pool_inst_mut,
        pool_bb_mut,
        pool_func_mut,
    };
    compunit.process(1, &mut kit_mut);
}

pub enum InfuncChoice {
    InFunc(&'static mut BasicBlock),
    NInFunc(),
}

pub trait Process {
    type Ret;
    type Message;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error>;
}

impl Process for CompUnit {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        for item in &mut self.global_items {
            item.process(1, kit_mut);
        }
        Err(Error::Todo)
    }
}

impl Process for GlobalItems {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::Decl(decl) => {
                decl.process(1, kit_mut);
            }
            Self::FuncDef(funcdef) => {
                funcdef.process(true, kit_mut);
            }
        }
        Err(Error::Todo)
    }
}

impl Process for Decl {
    type Ret = i32;
    type Message = (i32);

    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::ConstDecl(_decl) => {}
            Self::VarDecl(vardef) => match vardef.btype {
                BType::Int => {
                    for def in &mut vardef.var_def_vec {
                        match def {
                            VarDef::NonArrayInit((id, val)) => match val {
                                InitVal::Exp(exp) => {
                                    let inst_ptr = exp.process(input, kit_mut);
                                }
                                InitVal::InitValVec(val_vec) => {}
                            },
                            VarDef::NonArray(id) => {
                                kit_mut.add_var(id.as_str(), Type::Int, false, vec![]);
                            }
                            VarDef::ArrayInit((id, exp_vec, val)) => {}
                            VarDef::Array((id, exp_vec)) => {
                                kit_mut.add_var(id.as_str(), Type::Int, true, vec![]);
                            }
                        }
                    }
                }
                BType::Float => {
                    for def in &mut vardef.var_def_vec {
                        match def {
                            VarDef::NonArrayInit((id, val)) => {}
                            VarDef::NonArray((id)) => {
                                kit_mut.add_var(id.as_str(), Type::Float, false, vec![]);
                            }
                            VarDef::ArrayInit((id, exp_vec, val)) => {}
                            VarDef::Array((id, exp_vec)) => {
                                kit_mut.add_var(id.as_str(), Type::Float, true, vec![]);
                            }
                        }
                    }
                }
            },
        }
        Err(Error::Todo)
    }
}

impl Process for ConstDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for BType {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for ConstInitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for VarDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for VarDef {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for InitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for FuncDef {
    type Ret = i32;
    type Message = bool;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::NonParameterFuncDef((tp, id, blk)) => {
                let func_ptr = kit_mut.pool_func_mut.new_function();
                let func_mut = func_ptr.as_mut();
                let bb = kit_mut.pool_bb_mut.put(BasicBlock::new("test_first"));
                func_mut.insert_first_bb(bb);
                match tp {
                    FuncType::Void => func_mut.set_return_type(IrType::Void),
                    FuncType::Int => func_mut.set_return_type(IrType::Int),
                    FuncType::Float => func_mut.set_return_type(IrType::Float),
                }
                kit_mut.context_mut.bb_now_set(bb.as_mut());
                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                blk.process(1, kit_mut);
            }
            Self::ParameterFuncDef(pf) => {}
        }
        // module.push_function(name, function);
        Err(Error::Todo)
    }
}

impl Process for FuncType {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for FuncFParams {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for FuncFParam {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for Block {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        kit_mut.context_mut.add_layer();
        for item in &mut self.block_vec {
            item.process(input, kit_mut);
        }
        kit_mut.context_mut.delete_layer();
        Err(Error::Todo)
    }
}

impl Process for BlockItem {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            BlockItem::Decl(decl) => {
                decl.process(input, kit_mut);
            }
            BlockItem::Stmt(stmt) => {
                stmt.process(input, kit_mut);
            }
        }
        Err(Error::Todo)
    }
}
impl Process for Stmt {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Stmt::Assign(assign) => {}
            Stmt::ExpStmt(exp_stmt) => {}
            Stmt::Block(blk) => {}
            Stmt::If(if_stmt) => {}
            Stmt::While(while_stmt) => {}
            Stmt::Break(break_stmt) => {}
            Stmt::Continue(continue_stmt) => {}
            Stmt::Return(ret_stmt) => {}
        }
        Err(Error::Todo)
    }
}

impl Process for Assign {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for ExpStmt {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for If {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for While {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for Break {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for Continue {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for Return {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for Exp {
    type Ret = ObjPtr<Inst>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.add_exp.process(input, kit_mut)
    }
}

impl Process for Cond {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for LVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for PrimaryExp {
    type Ret = ObjPtr<Inst>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            PrimaryExp::Exp(exp) => exp.process(input, kit_mut),
            PrimaryExp::LVal(lval) => Err(Error::Todo),
            PrimaryExp::Number(num) => num.process(input, kit_mut),
        }
        // Err(Error::Todo)
    }
}
impl Process for Number {
    type Ret = ObjPtr<Inst>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Number::FloatConst(f) => {
                if let Some(inst) = kit_mut.context_mut.get_const_float(*f) {
                    return Ok(inst);
                } else {
                    let inst = kit_mut.pool_inst_mut.make_float_const(*f);
                    kit_mut.context_mut.add_const_float(*f, inst);
                    return Ok(inst);
                }
            }
            Number::IntConst(i) => {
                if let Some(inst) = kit_mut.context_mut.get_const_int(*i) {
                    return Ok(inst);
                } else {
                    let inst = kit_mut.pool_inst_mut.make_int_const(*i);
                    kit_mut.context_mut.add_const_int(*i, inst);
                    return Ok(inst);
                }
            }
        }
    }
}

impl Process for OptionFuncFParams {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for UnaryExp {
    type Ret = ObjPtr<Inst>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            UnaryExp::PrimaryExp(primaryexp) => {
                let inst_ptr = primaryexp.process(input, kit_mut);
                Err(Error::Todo)
            }
            UnaryExp::OpUnary((unaryop, unaryexp)) => match unaryop {
                UnaryOp::Add => {
                    let inst_u = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_pos(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok(inst)
                }
                UnaryOp::Minus => {
                    let inst_u = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_neg(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok(inst)
                }
                UnaryOp::Exclamation => {
                    let inst_u = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_not(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok(inst)
                }
            },
            UnaryExp::FuncCall((funcname, funcparams)) => Err(Error::Todo),
            _ => unreachable!(),
        }
    }
}

impl Process for UnaryOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for FuncRParams {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for MulExp {
    type Ret = ObjPtr<Inst>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            MulExp::UnaryExp(unaryexp) => unaryexp.process(input, kit_mut),
            MulExp::MulExp((mulexp, unaryexp)) => {
                let inst_left = mulexp.as_mut().process(input, kit_mut).unwrap();
                let inst_right = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_mul(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                Ok(inst)
            }
            MulExp::DivExp((mulexp, unaryexp)) => {
                let inst_left = mulexp.as_mut().process(input, kit_mut).unwrap();
                let inst_right = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_div(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                Ok(inst)
            }
            MulExp::ModExp((mulexp, unaryexp)) => {
                let inst_left = mulexp.as_mut().process(input, kit_mut).unwrap();
                let inst_right = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_rem(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                Ok(inst)
            }
        }
    }
}
impl Process for AddOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for AddExp {
    type Ret = ObjPtr<Inst>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            AddExp::MulExp(mulexp) => mulexp.as_mut().process(input, kit_mut),
            AddExp::OpExp((opexp, op, mulexp)) => match op {
                AddOp::Add => {
                    let inst_left = opexp.process(input, kit_mut).unwrap();
                    let inst_right = opexp.process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok(inst)
                }
                AddOp::Minus => {
                    let inst_left = opexp.process(input, kit_mut).unwrap();
                    let inst_right = opexp.process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok(inst)
                }
            },
        }
    }
}
impl Process for RelOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for RelExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for EqExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for LAndExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
impl Process for ConstExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}

impl Process for LOrExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Err(Error::Todo)
    }
}
