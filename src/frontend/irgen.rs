use super::actionscope::Type;
use super::{actionscope::ActionScope, ast::*};
use crate::frontend::error::Error;
use crate::global_lalrpop::{IN_FUNC, MODULE};
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::ir_type::IrType;
use crate::ir::module::{self, Module};
use crate::utility::{ObjPool, ObjPtr};
use std::borrow::Borrow;
use std::thread::LocalKey;
use std::{self};

struct Kit {
    module_mut: &'static mut Module,
    scope_mut: &'static mut ActionScope,
    pool_inst_mut: &'static mut ObjPool<Inst>,
    pool_func_mut: &'static mut ObjPool<Function>,
    pool_bb_mut: &'static mut ObjPool<BasicBlock>,
    bb_now_mut: InfuncChoice,
}

impl Kit {
    pub fn push_inst(&self, inst_ptr: ObjPtr<Inst>) {
        match self.bb_now_mut {
            InfuncChoice::InFunc(bb) => bb.push_back(inst_ptr),
            InfuncChoice::NInFunc() => {}
        }
    }

    // pub get_layer(&self)->i64{
    //     return self.scope_
    // }

    pub fn add_var(&self, s: &'static str, tp: Type, is_array: bool, dimension: Vec<i64>) {
        self.scope_mut.add_var(s, tp, is_array, dimension);
    }

    pub fn update_var(&self, s: &str, bbname: &str, inst: ObjPtr<Inst>) -> bool {
        self.update_var(s, bbname, inst)
    }

    pub fn push_globalvar(&self, name: &'static str, inst_ptr: ObjPtr<Inst>) {
        match self.bb_now_mut {
            InfuncChoice::InFunc(bb) => {}
            InfuncChoice::NInFunc() => self.module_mut.push_var(name, inst_ptr),
        }
    }

    pub fn push_function(&self, name: &'static str, func_ptr: ObjPtr<Function>) {
        match self.bb_now_mut {
            InfuncChoice::InFunc(bb) => {}
            InfuncChoice::NInFunc() => self.module_mut.push_function(name, func_ptr),
        }
    }

    pub fn bb_now_set(&self, bb: &mut BasicBlock) {
        self.bb_now_mut = InfuncChoice::InFunc(bb);
    }
}

pub fn irgen(
    compunit: &mut CompUnit,
    module_mut: &mut Module,
    pool_inst_mut: &mut ObjPool<Inst>,
    pool_bb_mut: &mut ObjPool<BasicBlock>,
    pool_func_mut: &mut ObjPool<Function>,
) {
    let mut pool_scope = ObjPool::new();
    let scope_mut = pool_scope.put(ActionScope::new()).as_mut();
    let s = "0".to_string();
    let bb_head = pool_bb_mut.put(BasicBlock::new(s.as_str()));
    let bb_now_mut = bb_head.clone();
    let mut kit_mut = &mut Kit {
        module_mut,
        scope_mut,
        pool_inst_mut,
        pool_bb_mut,
        pool_func_mut,
        bb_now_mut: InfuncChoice::NInFunc(),
    };
    compunit.process(1, kit_mut);
}

pub enum InfuncChoice {
    InFunc(&'static mut BasicBlock),
    NInFunc(),
}

// pub trait Process {
//     type Ret;
//     type Message;
//     fn process(
//         &self,
//         input: Self::Message,
//         module: &mut Module,
//         scope: &mut ActionScope,
//         pool_inst: &mut ObjPool<Inst>,
//         pool_bb: &mut ObjPool<BasicBlock>,
//         bb_now_mut: ObjPtr<BasicBlock>,
//         // pool: ObjPool<>,
//     ) -> Result<Self::Ret, Error>;
// }

// impl Process for CompUnit {
//     type Ret = i32;
//     type Message = (i32);
//     fn process(
//         &self,
//         input: Self::Message,
//         module: &mut Module,
//         scope: &mut ActionScope,
//         pool_inst: &mut ObjPool<Inst>,
//         pool_bb: &mut ObjPool<BasicBlock>,
//         bb_now_mut: ObjPtr<BasicBlock>,
//     ) -> Result<Self::Ret, Error> {
//         for item in self.global_items {
//             item.process(1, module, scope, pool_inst, pool_bb, bb_now_mut);
//         }
//         Ok(1)
//     }
// }

pub trait Process {
    type Ret;
    type Message;
    fn process(
        &self,
        input: Self::Message,
        kit_mut: &mut Kit,
        // pool: ObjPool<>,
    ) -> Result<Self::Ret, Error>;
}

impl Process for CompUnit {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        for item in self.global_items {
            item.process(1, kit_mut);
        }
        Ok(1)
    }
}

impl Process for GlobalItems {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::Decl(decl) => {
                decl.process(1, kit_mut);
            }
            Self::FuncDef(funcdef) => {
                funcdef.process(true, kit_mut);
            }
        }
        Ok(1)
    }
}

impl Process for Decl {
    type Ret = i32;
    type Message = (i32);

    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // let in_func = input;
        // match in_func {
        //     InfuncChoice::InFunc(bb) =>
        match self {
            Self::ConstDecl(decl) => {}
            Self::VarDecl(vardef) => match vardef.btype {
                BType::Int => {
                    for def in vardef.var_def_vec {
                        match def {
                            VarDef::NonArrayInit((id, val)) => match val {
                                InitVal::Exp(exp) => {}
                                InitVal::InitValVec(val_vec) => {}
                            },
                            VarDef::NonArray((id)) => {
                                // let inst_ptr = pool_inst.put(Inst::make_int_const(0));
                                kit_mut.add_var(id.as_str(), Type::Int, false, vec![]);
                                // let inst_ptr = kit_mut.pool_inst_mut.make_global_int(0);
                                // bb.push_back(inst.clone());
                                // kit_mut.push_inst(inst_ptr);
                            }
                            VarDef::ArrayInit((id, exp_vec, val)) => {}
                            VarDef::Array((id, exp_vec)) => {
                                kit_mut.add_var(id.as_str(), Type::Int, true, vec![]);
                            }
                        }
                    }
                }
                BType::Float => {
                    for def in vardef.var_def_vec {
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
        Ok(1)
    }
}

impl Process for ConstDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for BType {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for ConstInitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for VarDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for VarDef {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for InitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for FuncDef {
    type Ret = i32;
    type Message = bool;
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
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
                kit_mut.bb_now_set(bb.as_mut());
                kit_mut.push_function(id.as_str(), func_ptr);
                blk.process(1, kit_mut);
            }
            Self::ParameterFuncDef(pf) => {}
        }
        // module.push_function(name, function);
        Ok(1)
    }
}

impl Process for FuncType {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for FuncFParams {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for FuncFParam {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Block {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        kit_mut.scope_mut.add_layer();
        for item in self.block_vec {
            item.process(input, kit_mut);
        }
        kit_mut.scope_mut.delete_layer();
        Ok(1)
    }
}

impl Process for BlockItem {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            BlockItem::Decl(decl) => {
                decl.process(input, kit_mut);
            }
            BlockItem::Stmt(stmt) => {
                stmt.process(input, kit_mut);
            }
        }
        Ok(1)
    }
}
impl Process for Stmt {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
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
        Ok(1)
    }
}

impl Process for Assign {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for ExpStmt {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for If {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for While {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Break {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Continue {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Return {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Exp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Cond {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for LVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for PrimaryExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Number {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for OptionFuncFParams {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for UnaryExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for UnaryOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for FuncRParams {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for MulExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for AddOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for AddExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for RelOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for RelExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for EqExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for LAndExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for ConstExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for LOrExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
