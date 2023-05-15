use super::{actionscope::ActionScope, ast::*};
use crate::frontend::error::Error;
use crate::global_lalrpop::{IN_FUNC, MODULE};
use crate::ir::function::Function;
use crate::ir::module::{self, Module};
use std::borrow::Borrow;
use std::thread::LocalKey;
use std::{self};
pub fn irgen(compunit: &mut CompUnit) {
    let mut module = Module::make_module();
    let mut scope = ActionScope::new();
    compunit.process(1, &mut module, &mut scope);
}
pub trait Process {
    type Ret;
    type Message;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error>;
}

impl Process for CompUnit {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for GlobalItems {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        match self {
            Self::Decl(decl) => {}
            Self::FuncDef(funcdef) => {
                IN_FUNC.with(|i| {
                    let mut valtemp = i.borrow_mut();
                    *valtemp = 1;
                });
                funcdef.process(1, module, scope);
                IN_FUNC.with(|i| {
                    let mut valtemp = i.borrow_mut();
                    *valtemp = 0;
                });
            }
        }
        Ok(1)
    }
}

impl Process for Decl {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        match self {
            Self::ConstDecl(decl) => {}
            Self::VarDecl(funcdef) => {}
        }
        Ok(1)
    }
}

impl Process for ConstDecl {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for BType {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for ConstInitVal {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for VarDecl {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for VarDef {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for InitVal {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for FuncDef {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
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
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for FuncFParams {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for FuncFParam {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Block {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for BlockItem {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Stmt {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Assign {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for ExpStmt {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for If {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for While {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Break {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Continue {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Return {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Exp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for Cond {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for LVal {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for PrimaryExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for Number {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for OptionFuncFParams {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for UnaryExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for UnaryOp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for FuncRParams {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for MulExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for AddOp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for AddExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for RelOp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for RelExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for EqExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for LAndExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
impl Process for ConstExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for LOrExp {
    type Ret = i32;
    type Message = i32;
    fn process(
        &self,
        input: Self::Message,
        module: &mut Module,
        scope: &mut ActionScope,
    ) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}
