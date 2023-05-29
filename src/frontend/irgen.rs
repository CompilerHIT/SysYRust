use std::env::var;

use super::context::Type;
use super::{ast::*, context::Context};
use crate::frontend::context::Symbol;
use crate::frontend::error::Error;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
use crate::utility::{ObjPool, ObjPtr};

pub struct Kit<'a> {
    context_mut: &'a mut Context<'a>,
    pool_inst_mut: &'a mut ObjPool<Inst>,
    pool_func_mut: &'a mut ObjPool<Function>,
    pool_bb_mut: &'a mut ObjPool<BasicBlock>,
}

impl Kit<'_> {
    pub fn push_inst(&mut self, inst_ptr: ObjPtr<Inst>) {
        self.context_mut.push_inst_bb(inst_ptr);
    }

    pub fn add_var(&mut self, s: &str, tp: Type, is_array: bool, dimension: Vec<i64>) {
        self.context_mut.add_var(s, tp, is_array, dimension);
    }

    // pub fn update_var(&mut self, s: &str, inst: ObjPtr<Inst>) -> bool {
    //     self.context_mut.update_var_scope(s, inst)
    // }

    pub fn push_phi(
        &mut self,
        name: String,
        infunchoice: InfuncChoice,
    ) -> Result<ObjPtr<Inst>, Error> {
        match infunchoice {
            InfuncChoice::InFunc(bbptr) => {
                let bb = bbptr.as_mut();
                let inst_ptr = self.pool_inst_mut.make_float_phi();
                bb.push_front(inst_ptr);
                self.context_mut.update_var_scope(
                    name.as_str(),
                    inst_ptr,
                    InfuncChoice::InFunc(bbptr),
                );
                Ok(inst_ptr)
            }
            // InfuncChoice::NInFunc() => self.module_mut.push_var(name, inst_ptr),
            InfuncChoice::NInFunc() => Err(Error::PushPhiInGlobalDomain),
        }
    }

    pub fn get_var_symbol(&mut self,s:&str) ->Result<Symbol,Error>{
        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, _)| self.context_mut.symbol_table.get(last_elm))
            .map(|x| x.clone());
        if let Some(sym) = sym_opt{
            return Ok(sym);
        }
        Err(Error::VariableNotFound)
    }

    pub fn get_var(&mut self, s: &str) -> Result<(ObjPtr<Inst>, Symbol), Error> {
        // let bb = self.context_mut.bb_now_mut;
        match self.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb) => {
                if let Some((inst, symbol)) = self.get_var_bb(s, bb) {
                    return Ok((inst, symbol));
                }
            }
            _ => todo!(),
            // InfuncChoice::NInFunc() => {
            //     return todo!();;
            // }
        }
        return Err(Error::VariableNotFound);
    }

    pub fn get_var_bb(
        &mut self,
        s: &str,
        bb: ObjPtr<BasicBlock>,
    ) -> Option<(ObjPtr<Inst>, Symbol)> {
        // if let Some(vec_temp) = self.context_mut.var_map.get(s.clone()) {
        //     if let Some((last_element0, _last_element1)) = vec_temp.last() {
        //         let name = last_element0.clone();
        //         if let Some(symbol_temp) = self.context_mut.symbol_table.get(&name) {
        //             let symbol = symbol_temp.clone();

        //             let bbname = bb.as_ref().get_name();
        //             if let Some(var_inst_map) = self.context_mut.bb_map.get(bbname) {
        //                 if let Some(inst) = var_inst_map.get(s.clone()) {
        //                     return Option::Some((*inst, symbol));
        //                 }
        //             }
        //         }
        //     }
        // }

        let mut name_changed = " ".to_string();

        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, _)| {name_changed = last_elm.clone();self.context_mut.symbol_table.get(last_elm)})
            .map(|x| x.clone());

        let inst_opt = self
            .context_mut
            .bb_map
            .get(bb.as_ref().get_name())
            .and_then(|var_inst_map| var_inst_map.get(&name_changed));

        if let Some(sym) = sym_opt {
            // println!("找到变量{:?}",s);
            if let Some(inst) = inst_opt {
                return Some((*inst, sym));
            } else {
                println!("没找到");
                //没找到
                let phiinst = self
                    .push_phi(s.to_string(), InfuncChoice::InFunc(bb))
                    .unwrap();
                // let phiinst_mut = phiinst.as_mut();
                // let bb_mut = bb.as_mut();
                // for preccessor in bb_mut.get_up_bb() {
                //     if let Some((temp, symbol)) = self.get_var_bb(s, *preccessor) {
                //         phiinst_mut.add_operand(temp);
                //     }
                // }
                return Option::Some((phiinst, sym));
            }
        }
        Option::None
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
    let context_mut = pool_scope.put(Context::make_context(module_mut)).as_mut();
    let mut kit_mut = Kit {
        context_mut,
        pool_inst_mut,
        pool_bb_mut,
        pool_func_mut,
    };
    compunit.process(1, &mut kit_mut);
}

#[derive(Clone, Copy)]
pub enum InfuncChoice {
    InFunc(ObjPtr<BasicBlock>),
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
        return Ok(1);
        todo!();
    }
}

impl Process for GlobalItems {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::Decl(decl) => {
                decl.process(1, kit_mut);
                Ok(1)
            }
            Self::FuncDef(funcdef) => {
                funcdef.process(true, kit_mut);
                Ok(1)
            }
        }
        // todo!();
    }
}

impl Process for Decl {
    type Ret = i32;
    type Message = (i32);

    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::ConstDecl(constdecl) => {return constdecl.process(input, kit_mut);}
            Self::VarDecl(vardef) => {return vardef.process(input, kit_mut);}
        }
        todo!();
    }
}

impl Process for ConstDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self.btype {
            BType::Int => {
                for def in &mut self.const_def_vec {
                    
                    // match def {
                    //     VarDef::NonArrayInit((id, val)) => match val {
                    //         InitVal::Exp(exp) => {
                    //             let inst_ptr = exp.process(input, kit_mut).unwrap();
                    //             kit_mut
                    //                 .context_mut
                    //                 .add_var(id, Type::Int, false, Vec::new());
                    //             kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                    //             return Ok(1);
                    //         }
                    //         InitVal::InitValVec(val_vec) => {
                    //             todo!()
                    //         }
                    //     },
                    //     VarDef::NonArray(id) => {
                    //         kit_mut
                    //             .context_mut
                    //             .add_var(id, Type::Int, false, Vec::new());
                    //         return Ok(1);
                    //     }
                    //     VarDef::ArrayInit((id, exp_vec, val)) => {}
                    //     VarDef::Array((id, exp_vec)) => {
                    //         kit_mut
                    //             .context_mut
                    //             .add_var(id.as_str(), Type::Int, true, vec![]);
                    //         return Ok(1);
                    //     }
                    // }
                }
                Ok(1)
            }
            BType::Float => {
                for def in &mut self.const_def_vec {
                    // match def {
                    //     VarDef::NonArrayInit((id, val)) => match val {
                    //         InitVal::Exp(exp) => {
                    //             let inst_ptr = exp.process(input, kit_mut).unwrap();
                    //             kit_mut
                    //                 .context_mut
                    //                 .add_var(id, Type::Float, false, Vec::new());
                    //             kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                    //             return Ok(1);
                    //         }
                    //         InitVal::InitValVec(val_vec) => {
                    //             todo!()
                    //         }
                    //     },
                    //     VarDef::NonArray((id)) => {
                    //         kit_mut.add_var(id.as_str(), Type::Float, false, vec![]);
                    //         return Ok(1);
                    //     }
                    //     VarDef::ArrayInit((id, exp_vec, val)) => {
                    //         todo!()
                    //     }
                    //     VarDef::Array((id, exp_vec)) => {
                    //         // kit_mut.add_var(id.as_str(), Type::Float, true, vec![]);
                    //         // return Ok(1);
                    //     }
                    //     _ => todo!(),
                    // }
                }
                Ok(1)
            }
            
        }
    }
}

// impl Process for BType {
//     type Ret = i32;
//     type Message = (i32);
//     fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
//         todo!();
//     }
// }

impl Process for ConstInitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for VarDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self.btype {
            BType::Int => {
                for def in &mut self.var_def_vec {
                    match def {
                        VarDef::NonArrayInit((id, val)) => match val {
                            InitVal::Exp(exp) => {
                                let inst_ptr = exp.process(Type::Int, kit_mut).unwrap();
                                kit_mut
                                    .context_mut
                                    .add_var(id, Type::Int, false, Vec::new());
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                                
                            }
                            InitVal::InitValVec(val_vec) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray(id) => {
                            kit_mut
                                .context_mut
                                .add_var(id, Type::Int, false, Vec::new());
                            
                        }
                        VarDef::ArrayInit((id, exp_vec, val)) => {}
                        VarDef::Array((id, exp_vec)) => {
                            kit_mut
                                .context_mut
                                .add_var(id.as_str(), Type::Int, true, vec![]);
                            
                        }
                    }
                }
                Ok(1)
            }
            BType::Float => {
                for def in &mut self.var_def_vec {
                    match def {
                        VarDef::NonArrayInit((id, val)) => match val {
                            InitVal::Exp(exp) => {
                                let inst_ptr = exp.process(Type::Float, kit_mut).unwrap();
                                kit_mut
                                    .context_mut
                                    .add_var(id, Type::Float, false, Vec::new());
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                                
                            }
                            InitVal::InitValVec(val_vec) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray((id)) => {
                            kit_mut.add_var(id.as_str(), Type::Float, false, vec![]);
                            
                        }
                        VarDef::ArrayInit((id, exp_vec, val)) => {
                            todo!()
                        }
                        VarDef::Array((id, exp_vec)) => {
                            // kit_mut.add_var(id.as_str(), Type::Float, true, vec![]);
                            // return Ok(1);
                        }
                        _ => todo!(),
                    }
                }
                Ok(1)
            }
            
        }
    }
}
impl Process for VarDef {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for InitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
impl Process for FuncDef {
    type Ret = i32;
    type Message = bool;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::NonParameterFuncDef((tp, id, blk)) => {
                kit_mut.context_mut.add_layer();
                let func_ptr = kit_mut.pool_func_mut.new_function();
                let func_mut = func_ptr.as_mut();
                let bb = kit_mut.pool_bb_mut.new_basic_block(id.clone());
                func_mut.insert_first_bb(bb);
                match tp {
                    FuncType::Void => func_mut.set_return_type(IrType::Void),
                    FuncType::Int => func_mut.set_return_type(IrType::Int),
                    FuncType::Float => func_mut.set_return_type(IrType::Float),
                }
                kit_mut.context_mut.bb_now_set(bb);
                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                blk.process(1, kit_mut);
                kit_mut.context_mut.delete_layer();
                return Ok(1);
            }
            Self::ParameterFuncDef((tp, id, params, blk)) => {
                kit_mut.context_mut.add_layer();
                let func_ptr = kit_mut.pool_func_mut.new_function();
                let func_mut = func_ptr.as_mut();
                let bb = kit_mut.pool_bb_mut.new_basic_block(id.clone());
                func_mut.insert_first_bb(bb);
                match tp {
                    FuncType::Void => func_mut.set_return_type(IrType::Void),
                    FuncType::Int => func_mut.set_return_type(IrType::Int),
                    FuncType::Float => func_mut.set_return_type(IrType::Float),
                }
                kit_mut.context_mut.bb_now_set(bb);
                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                let params_vec = params.process(1, kit_mut).unwrap();
                for (name, param) in params_vec {
                    func_mut.set_parameter(name, param); //这里
                }
                blk.process(1, kit_mut);
                kit_mut.context_mut.delete_layer();
                return Ok(1);
            }
        }
        // module.push_function(name, function);
        todo!();
    }
}

// impl Process for FuncType {
//     type Ret = i32;
//     type Message = (i32);
//     fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
//         todo!();
//     }
// }
impl Process for FuncFParams {
    type Ret = Vec<(String, ObjPtr<Inst>)>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let mut vec = vec![];
        for param in &mut self.func_fparams_vec {
            let p = param.process(input, kit_mut).unwrap();
            vec.push(p);
        }
        Ok(vec)
    }
}

impl Process for FuncFParam {
    type Ret = (String, ObjPtr<Inst>);
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            FuncFParam::Array((tp, id, vec)) => {
                todo!()
            }
            // BType::Int => {}
            // BType::Float => {}
            // todo!();
            // },
            FuncFParam::NonArray((tp, id)) => match tp {
                BType::Int => {
                    let param = kit_mut.pool_inst_mut.make_param(IrType::Int);
                    kit_mut
                        .context_mut
                        .add_var(id, Type::Int, false, Vec::new());
                    //这里
                    // kit_mut.context_mut.update_var_scope_now(s, inst)
                    Ok((id.clone(), param))
                }
                BType::Float => {
                    let param = kit_mut.pool_inst_mut.make_param(IrType::Float);
                    kit_mut
                        .context_mut
                        .add_var(id, Type::Float, false, Vec::new());
                    Ok((id.clone(), param))
                }
            },
        }
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
        Ok(1)
    }
}

impl Process for BlockItem {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            BlockItem::Decl(decl) => {
                decl.process(input, kit_mut);
                return Ok(1);
            }
            BlockItem::Stmt(stmt) => {
                stmt.process(input, kit_mut);
                return Ok(1);
            }
        }
        todo!();
    }
}
impl Process for Stmt {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Stmt::Assign(assign) => {
                assign.process(input, kit_mut);
                Ok(1)
            }
            Stmt::ExpStmt(exp_stmt) => {
                exp_stmt.process(Type::Int, kit_mut);//这里可能有问题
                Ok(1)
            }
            Stmt::Block(blk) => {
                blk.process(input, kit_mut);
                Ok(1)
            }
            Stmt::If(if_stmt) => {
                if_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::While(while_stmt) => {
                while_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::Break(break_stmt) => {
                break_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::Continue(continue_stmt) => {
                continue_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::Return(ret_stmt) => {
                ret_stmt.process(input, kit_mut);
                Ok(1)
            }
        }
        // todo!();
    }
}

impl Process for Assign {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let lval = &mut self.lval;
        let symbol = kit_mut.get_var_symbol(&lval.id).unwrap();
        // let (_,symbol) = kit_mut.get_var(&lval.id).unwrap();
        println!("assign stmt");
        let mut mes = Type::Int;
        match symbol.tp{
            Type::ConstFloat =>{mes = Type::Float;}
            Type::ConstInt =>{mes = Type::Int;}
            Type::Float =>{mes = Type::Float;}
            Type::Int =>{mes = Type::Int;}
        }
        let inst_r = self.exp.process(mes, kit_mut).unwrap();
        kit_mut
            .context_mut
            .update_var_scope_now(&self.lval.id, inst_r);
        Ok(1)
    }
}
impl Process for ExpStmt {
    type Ret = i32;
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for If {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
impl Process for While {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for Break {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
impl Process for Continue {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for Return {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if let Some(exp) = &mut self.exp {
            let inst = exp.process(Type::Int, kit_mut).unwrap();//这里可能有问题
            let ret_inst = kit_mut.pool_inst_mut.make_return(inst);
            kit_mut.context_mut.push_inst_bb(ret_inst);
            Ok(1)
        } else {
            // let ret_inst = kit_mut.pool_inst_mut.make_return(inst);
            // kit_mut.context_mut.push_inst_bb(ret_inst);
            // Ok(1)
            todo!()
        }
    }
}
impl Process for Exp {
    type Ret = ObjPtr<Inst>;
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.add_exp.process(input, kit_mut)
    }
}

impl Process for Cond {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
impl Process for LVal {
    type Ret = ObjPtr<Inst>;
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // let id = self.id;
        // let vec = self.exp_vec;
        let (var, symbol) = kit_mut.get_var(&self.id).unwrap();
        println!("var_name:{:?},ir_type:{:?}",&self.id,var.as_ref().get_ir_type());
        println!("var_name:{:?},ir_type:{:?}",&self.id,var.as_ref().get_kind());
        if symbol.is_array {
            todo!();
        } else {
            return Ok(var);
        }
    }
}

impl Process for PrimaryExp {
    type Ret = ObjPtr<Inst>;
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            PrimaryExp::Exp(exp) => exp.process(input, kit_mut),
            PrimaryExp::LVal(lval) => lval.process(input, kit_mut),
            PrimaryExp::Number(num) => num.process(input, kit_mut),
        }
        // todo!();
    }
}
impl Process for Number {
    type Ret = ObjPtr<Inst>;
    type Message = (Type);
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
                    // println!("intconst:{}", i);
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
        todo!();
    }
}
impl Process for UnaryExp {
    type Ret = ObjPtr<Inst>;
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            UnaryExp::PrimaryExp(primaryexp) => primaryexp.process(input, kit_mut),
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
            UnaryExp::FuncCall((funcname, funcparams)) => todo!(),
            _ => unreachable!(),
        }
    }
}

impl Process for UnaryOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for FuncRParams {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // match self{
        //     FuncFParams
        // }
        todo!()
    }
}

impl Process for MulExp {
    type Ret = ObjPtr<Inst>;
    type Message = (Type);
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
        todo!();
    }
}

impl Process for AddExp {
    type Ret = ObjPtr<Inst>;
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            AddExp::MulExp(mulexp) => mulexp.as_mut().process(input, kit_mut),
            AddExp::OpExp((opexp, op, mulexp)) => match op {
                AddOp::Add => {
                    let inst_left = opexp.process(input, kit_mut).unwrap();
                    let inst_right = mulexp.process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok(inst)
                }
                AddOp::Minus => {
                    let inst_left = opexp.process(input, kit_mut).unwrap();
                    let inst_right = mulexp.process(input, kit_mut).unwrap();
                    let inst_right_neg = kit_mut.pool_inst_mut.make_neg(inst_right);
                    let inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right_neg);
                    kit_mut.context_mut.push_inst_bb(inst_right_neg);
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
        todo!();
    }
}

impl Process for RelExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
impl Process for EqExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for LAndExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
impl Process for ConstExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for LOrExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
