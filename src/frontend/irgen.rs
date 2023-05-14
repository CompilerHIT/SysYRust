use super::{actionscope::ActionScope, ast::*};
use crate::frontend::error::Error;
use crate::ir::module::{self, Module};
use std::{self};
pub fn irgen(compunit: &mut CompUnit) {}
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

impl Process for GlobalItems {
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

impl Process for Decl {
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
