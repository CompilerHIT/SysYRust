use super::context::Type;
use super::{ast::*, kit::Kit};
use crate::frontend::error::Error;
use crate::ir::ir_type::IrType;
pub trait TypeProcess {
    type Ret; //i32,3代表Float,2代表ImmFloat，1代表Int，0代表ImmInt
    type Message;
    fn type_process(&mut self, input: Self::Message, kit_mut: &mut Kit)
        -> Result<Self::Ret, Error>;
}

impl TypeProcess for RelExp {
    type Ret = i32;
    type Message = i32;
    fn type_process(
        &mut self,
        input: Self::Message,
        kit_mut: &mut Kit,
    ) -> Result<Self::Ret, Error> {
        match self {
            RelExp::AddExp(addexp) => {
                return addexp.type_process(input, kit_mut);
            }
            RelExp::OpExp((_, _, _)) => {
                return Ok(1);
            }
        }
    }
}

impl TypeProcess for AddExp {
    type Ret = i32;
    type Message = i32;
    fn type_process(
        &mut self,
        input: Self::Message,
        kit_mut: &mut Kit,
    ) -> Result<Self::Ret, Error> {
        match self {
            AddExp::MulExp(mulexp) => return mulexp.type_process(input, kit_mut),
            AddExp::OpExp((addexp, op, mulexp)) => {
                let tp_left = addexp.type_process(input, kit_mut).unwrap();
                let tp_right = mulexp.type_process(input, kit_mut).unwrap();
                if tp_left > tp_right {
                    return Ok(tp_left);
                } else {
                    return Ok(tp_right);
                }
            }
        }
    }
}

impl TypeProcess for MulExp {
    type Ret = i32;
    type Message = i32;
    fn type_process(
        &mut self,
        input: Self::Message,
        kit_mut: &mut Kit,
    ) -> Result<Self::Ret, Error> {
        match self {
            MulExp::UnaryExp(unaryexp) => {
                return unaryexp.type_process(input, kit_mut);
            }
            MulExp::MulExp((mulexp, unaryexp))
            | MulExp::DivExp((mulexp, unaryexp))
            | MulExp::ModExp((mulexp, unaryexp)) => {
                let tp_left = mulexp.type_process(input, kit_mut).unwrap();
                let tp_right = unaryexp.type_process(input, kit_mut).unwrap();
                if tp_left > tp_right {
                    return Ok(tp_left);
                } else {
                    return Ok(tp_right);
                }
            }
        }
    }
}

impl TypeProcess for UnaryExp {
    type Ret = i32;
    type Message = i32;
    fn type_process(
        &mut self,
        input: Self::Message,
        kit_mut: &mut Kit,
    ) -> Result<Self::Ret, Error> {
        match self {
            UnaryExp::PrimaryExp(primaryexp) => primaryexp.type_process(input, kit_mut),
            UnaryExp::FuncCall((id, _)) => {
                let inst_func = kit_mut.context_mut.module_mut.get_function(&id);
                match inst_func.as_ref().get_return_type() {
                    IrType::Float => Ok(3),
                    IrType::Int => Ok(1),
                    _ => {
                        unreachable!()
                    }
                }
            }
            UnaryExp::OpUnary((_, unaryexp)) => unaryexp.type_process(input, kit_mut),
        }
    }
}

impl TypeProcess for PrimaryExp {
    type Ret = i32;
    type Message = i32;
    fn type_process(
        &mut self,
        input: Self::Message,
        kit_mut: &mut Kit,
    ) -> Result<Self::Ret, Error> {
        match self {
            PrimaryExp::Exp(exp) => exp.type_process(input, kit_mut),
            PrimaryExp::LVal(lval) => {
                let sym = kit_mut.get_var_symbol(&lval.id).unwrap();
                match sym.tp {
                    Type::ConstFloat | Type::Float => {
                        Ok(3)
                    }
                    Type::ConstInt | Type::Int => Ok(1),
                    _ => {
                        todo!()
                    }
                }
            }
            PrimaryExp::Number(imm) => {
                match imm {
                    Number::FloatConst(_) => {
                        //需要改吗优先级
                        Ok(2)
                    }
                    Number::IntConst(_) => Ok(0),
                }
            }
        }
    }
}

impl TypeProcess for Exp {
    type Ret = i32;
    type Message = i32;
    fn type_process(
        &mut self,
        input: Self::Message,
        kit_mut: &mut Kit,
    ) -> Result<Self::Ret, Error> {
        self.add_exp.type_process(input, kit_mut)
   }
}

impl TypeProcess for EqExp {
    type Ret = i32;
    type Message = i32;
    fn type_process(
        &mut self,
        input: Self::Message,
        kit_mut: &mut Kit,
    ) -> Result<Self::Ret, Error> {
        match self {
            EqExp::RelExp(relexp) => {
                return relexp.type_process(input, kit_mut);
            }
            EqExp::EqualExp((_, _)) | EqExp::NotEqualExp((_, _)) => {
                return Ok(1);
            }
        }
    }
}
