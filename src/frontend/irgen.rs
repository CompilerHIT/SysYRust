use super::{actionscope::ActionScope, ast::*};
use crate::frontend::error::Error;
use crate::global_lalrpop::{IN_FUNC, MODULE};
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::module::{self, Module};
use crate::utility::ObjPool;
use std::borrow::Borrow;
use std::thread::LocalKey;
use std::{self};

pub fn irgen(compunit: &mut CompUnit) {
    let mut pool_module = ObjPool::new();
    let module_ref = pool_module.put(Module::new()).as_mut();
    let mut pool_scope = ObjPool::new();
    let scope_ref = pool_scope.put(ActionScope::new()).as_mut();
    let mut pool_inst: ObjPool<Inst> = ObjPool::new();
    let mut pool_inst_ref = &mut pool_inst;
    compunit.process(1, module_ref, scope_ref,pool_inst_ref);
    // (module,pool_inst)
}

// pub struct Message{
//     in_func:bool,

// }



pub trait Process {
    type Ret;
    type Message;
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
        // pool: ObjPool<>,
    ) -> Result<Self::Ret, Error>;
}

impl Process for CompUnit {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        for item in self.global_items{
            item.process(1, module, scope,pool_inst);
        }
        Ok(1)
    }
}

impl Process for GlobalItems {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        match self {
            Self::Decl(decl) => {
                decl.process(false, module, scope,pool_inst);
            }
            Self::FuncDef(funcdef) => {
               funcdef.process(true, module, scope,pool_inst);
            }
        }
        Ok(1)
    }
}

impl Process for Decl {
    type Ret = i32;
    type Message = bool;
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        let in_func = input;
        if in_func{
            match self {
                Self::ConstDecl(decl) => {}
                Self::VarDecl(vardef) => {
                    match vardef.btype {
                        BType::Int =>{
                            for def in vardef.var_def_vec{
                                match def {
                                    VarDef::NonArrayInit((id,val)) =>{

                                    }
                                    VarDef::NonArray((id)) =>{

                                    }
                                    VarDef::ArrayInit((id,exp_vec,val)) =>{

                                    }
                                    VarDef::Array((id,exp_vec)) =>{

                                    }
                                }
                            }
                        }
                        BType::Float =>{
                            for def in vardef.var_def_vec{
                                match def {
                                    VarDef::NonArrayInit((id,val)) =>{

                                    }
                                    VarDef::NonArray((id)) =>{

                                    }
                                    VarDef::ArrayInit((id,exp_vec,val)) =>{

                                    }
                                    VarDef::Array((id,exp_vec)) =>{
                                        
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        else{
            match self {
                Self::ConstDecl(decl) => {}
                Self::VarDecl(vardef) => {
                    match vardef.btype {
                        BType::Int =>{
                            for def in vardef.var_def_vec{
                                match def {
                                    VarDef::NonArrayInit((id,val)) =>{
                                        // let var = pool_inst.put(Inst::make_global_int(val));
                                        // module.push_var(&id, var);
                                    }
                                    VarDef::NonArray((id)) =>{
                                        let var = pool_inst.put(Inst::make_global_int(0));
                                        module.push_var(&id, var);
                                    }
                                    VarDef::ArrayInit((id,exp_vec,val)) =>{

                                    }
                                    VarDef::Array((id,exp_vec)) =>{

                                    }
                                }
                            }
                        }
                        BType::Float =>{
                            for def in vardef.var_def_vec{
                                match def {
                                    VarDef::NonArrayInit((id,val)) =>{

                                    }
                                    VarDef::NonArray((id)) =>{

                                    }
                                    VarDef::ArrayInit((id,exp_vec,val)) =>{

                                    }
                                    VarDef::Array((id,exp_vec)) =>{
                                        
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(1)
    }
}

impl Process for ConstDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for BType {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for ConstInitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for VarDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for VarDef {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for InitVal {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for FuncDef {
    type Ret = i32;
    type Message = bool;
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        match self {
            Self::NonParameterFuncDef(npfd) => {
                // module.push_function(npfd.1, Function::make_function(head_block));
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
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for FuncFParams {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for FuncFParam {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Block {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for BlockItem {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Stmt {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Assign {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for ExpStmt {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for If {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for While {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Break {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Continue {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Return {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Exp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Cond {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for LVal {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for PrimaryExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Number {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for OptionFuncFParams {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for UnaryExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for UnaryOp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for FuncRParams {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for MulExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for AddOp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for AddExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for RelOp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for RelExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for EqExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for LAndExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for ConstExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for LOrExp {
    type Ret = i32;
    type Message = (i32);
    fn process(
        &self,
        input:Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
        pool_inst: &mut ObjPool<Inst>,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
