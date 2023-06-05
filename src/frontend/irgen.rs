use std::env::var;

use super::context::Type;
use super::ExpValue;
use super::{ast::*, context::Context};
use crate::frontend::context::Symbol;
use crate::frontend::error::Error;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::{self, Function};
use crate::ir::instruction::{Inst, InstKind};
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

    pub fn add_var(&mut self, s: &str, tp: Type, is_array: bool,is_param:bool, dimension: Vec<i64>) {
        self.context_mut.add_var(s, tp, is_array,is_param, dimension);
    }

    pub fn param_used(&mut self,s:&str){
        // let inst = self.context_mut.module_mut.get_var(s);

                let mut name_changed = " ".to_string();
                let mut layer_var = 0;

                self
                    .context_mut
                    .var_map
                    .get(s)
                    .and_then(|vec_temp| vec_temp.last())
                    .and_then(|(last_elm, layer)| {
                        name_changed = last_elm.clone();
                        layer_var = *layer;
                        self.context_mut.symbol_table.get(last_elm)
                    });//获得改名后的名字
                    
                self.context_mut.param_usage_table.insert(name_changed, true);
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

    pub fn get_var_symbol(&mut self, s: &str) -> Result<Symbol, Error> {
        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, _)| self.context_mut.symbol_table.get(last_elm))
            .map(|x| x.clone());
        if let Some(sym) = sym_opt {
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
            InfuncChoice::NInFunc() => {
                let inst = self.context_mut.module_mut.get_var(s);
                let mut name_changed = " ".to_string();
                let mut layer_var = 0;

                let sym_opt = self
                    .context_mut
                    .var_map
                    .get(s)
                    .and_then(|vec_temp| vec_temp.last())
                    .and_then(|(last_elm, layer)| {
                        name_changed = last_elm.clone();
                        layer_var = *layer;
                        self.context_mut.symbol_table.get(last_elm)
                    })
                    .map(|x| x.clone());
                let mut bbname = "notinblock";

                let inst_opt = self
                    .context_mut
                    .bb_map
                    .get(bbname)
                    .and_then(|var_inst_map| var_inst_map.get(&name_changed));

                if let Some(sym) = sym_opt {
                    // println!("找到变量{:?}",s);
                    return Ok((inst, sym));
                }
                // InfuncChoice::NInFunc() => {
                //     return todo!();;
                // }
            }
        }
        println!("没找到变量:{:?}",s);
        return Err(Error::VariableNotFound);
    }

    pub fn get_var_bb(
        &mut self,
        s: &str,
        bb: ObjPtr<BasicBlock>,
    ) -> Option<(ObjPtr<Inst>, Symbol)> {

        let mut name_changed = " ".to_string();
        let mut layer_var = 0;

        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, layer)| {
                name_changed = last_elm.clone();
                layer_var = *layer;
                self.context_mut.symbol_table.get(last_elm)
            })
            .map(|x| x.clone());

        let mut bbname = bb.as_ref().get_name();

        // let mut is_const = false;

        if layer_var == -1 {
            bbname = "notinblock"; //全局变量,const和一般类型需要分开处理吗?
        }else if layer_var ==0 {
            // bbname = "params";
            if let Some(is_used) = self.context_mut.param_usage_table.get(&name_changed){
                if !is_used{
                    // println!("进来了");
                    bbname = "params";
                }
                // bbname = "params";
            }
        }

        let inst_opt = self
            .context_mut
            .bb_map
            .get(bbname)
            .and_then(|var_inst_map| var_inst_map.get(&name_changed));

        if let Some(sym) = sym_opt {
            // println!("进来了");
            // println!("找到变量{:?}",s);
            if let Some(inst) = inst_opt {
                if layer_var < 0 {
                    // bbname = "notinblock";//全局变量,const和一般类型需要分开处理吗?
                    //这里返回一条load指令
                    match sym.tp {
                        Type::ConstFloat | Type::Float => {
                            let inst_load = self.pool_inst_mut.make_global_float_load(*inst);
                            self.context_mut.push_inst_bb(inst_load); //这里
                            return Some((inst_load, sym));
                        }
                        Type::ConstInt | Type::Int => {
                            let inst_load = self.pool_inst_mut.make_global_int_load(*inst);
                            self.context_mut.push_inst_bb(inst_load);
                            return Some((inst_load, sym));
                        }
                    }
                }
                return Some((*inst, sym));
            } else {
                println!("没找到变量{:?}", s);
                println!("bbname:{:?}",bbname);
                
                //没找到
                // bb.as_ref().
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
                decl.process(1, kit_mut).unwrap();
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
            Self::ConstDecl(constdecl) => {
                constdecl.process(input, kit_mut).unwrap();
                return Ok(1);
            }
            Self::VarDecl(vardef) => {
                vardef.process(input, kit_mut).unwrap();
                return Ok(1);
            }
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
                    if def.const_exp_vec.is_empty() {
                        let (mut inst_ptr, mut val) =
                            def.const_init_val.process(Type::ConstInt, kit_mut).unwrap();
                        if !kit_mut.context_mut.add_var(
                            &def.ident,
                            Type::ConstInt,
                            false,
                            false,
                            Vec::new(),
                        ) {
                            return Err(Error::MultipleDeclaration);
                        }

                        
                            let mut bond = 0;
                            match val {
                                ExpValue::Int(i) => {
                                    
                                    bond = i;
                                    if kit_mut.context_mut.get_layer() < 0 {
                                        inst_ptr = kit_mut.pool_inst_mut.make_global_int_const(bond);
                                    }else{
                                        inst_ptr = kit_mut.pool_inst_mut.make_int_const(i);
                                        kit_mut.context_mut.push_inst_bb(inst_ptr);
                                    }
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                            inst_ptr = kit_mut.pool_inst_mut.make_global_int_const(bond);
                            //这里
                        
                        kit_mut
                            .context_mut
                            .update_var_scope_now(&def.ident, inst_ptr);
                    } else {
                        todo!()
                    }
                }
                return Ok(1);
            }
            BType::Float => {
                for def in &mut self.const_def_vec {
                    if def.const_exp_vec.is_empty() {
                        let (mut inst_ptr, mut val) = def
                            .const_init_val
                            .process(Type::ConstFloat, kit_mut)
                            .unwrap();
                        if !kit_mut.context_mut.add_var(
                            &def.ident,
                            Type::ConstFloat,
                            false,
                            false,
                            Vec::new(),
                        ) {
                            return Err(Error::MultipleDeclaration);
                        }

                        // if kit_mut.context_mut.get_layer() < 0 {
                        //     let mut bond = 0.0;
                        //     match val {
                        //         ExpValue::Float(f) => {
                        //             bond = f;
                        //         }
                        //         _ => {
                        //             unreachable!()
                        //         }
                        //     }
                        //     inst_ptr = kit_mut.pool_inst_mut.make_global_float_const(bond);
                        //     //这里
                        // }else{
                        //     kit_mut.context_mut.push_inst_bb(inst_ptr);
                        // }

                        let mut bond= 0.0;
                            match val {
                                ExpValue::Float(i) => {
                                    
                                    bond = i;
                                    if kit_mut.context_mut.get_layer() < 0 {
                                        inst_ptr = kit_mut.pool_inst_mut.make_global_float_const(bond);
                                    }else{
                                        inst_ptr = kit_mut.pool_inst_mut.make_float_const(i);
                                        kit_mut.context_mut.push_inst_bb(inst_ptr);
                                    }
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                            inst_ptr = kit_mut.pool_inst_mut.make_global_float_const(bond);
                            //这里
                        kit_mut
                            .context_mut
                            .update_var_scope_now(&def.ident, inst_ptr);
                    } else {
                        todo!()
                    }
                }
                return Ok(1);
            }
        }
        Ok(1)
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
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            ConstInitVal::ConstExp(constexp) => constexp.process(input, kit_mut),
            ConstInitVal::ConstInitValVec(constvalvec) => {
                todo!()
            }
        }
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
                                let (mut inst_ptr, mut val) =
                                    exp.process(Type::Int, kit_mut).unwrap();
                                if !kit_mut
                                    .context_mut
                                    .add_var(id, Type::Int, false,false, Vec::new())
                                {
                                    return Err(Error::MultipleDeclaration);
                                }
                                if kit_mut.context_mut.get_layer() < 0 {//设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                    match val {
                                        ExpValue::Int(i) => {
                                            inst_ptr = kit_mut.pool_inst_mut.make_global_int(i);
                                            //这里
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
                            InitVal::InitValVec(val_vec) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray(id) => {
                            if !kit_mut
                                .context_mut
                                .add_var(id, Type::Int, false, false,Vec::new())
                            {
                                return Err(Error::MultipleDeclaration);
                            }
                            if kit_mut.context_mut.get_layer()==-1{//设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                let inst_ptr = kit_mut.pool_inst_mut.make_global_int(0);
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
                        }
                        VarDef::ArrayInit((id, exp_vec, val)) => {
                            // let dimension_vec = exp_vec.iter().map(|x|x.process(Type::Int, kit_mut).unwrap()).collect();
                            // val.process((Type::Int,), kit_mut).unwrap();
                        }
                        VarDef::Array((id, exp_vec)) => {
                            if !kit_mut
                                .context_mut
                                .add_var(id.as_str(), Type::Int, true,false, vec![])
                            {
                                return Err(Error::MultipleDeclaration);
                            }
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
                                let (mut inst_ptr, val) =
                                    exp.process(Type::Float, kit_mut).unwrap();
                                if !kit_mut
                                    .context_mut
                                    .add_var(id, Type::Float, false,false, Vec::new())
                                {
                                    return Err(Error::MultipleDeclaration);
                                }

                                if kit_mut.context_mut.get_layer() < 0 {//设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                    match val {
                                        ExpValue::Float(f) => {
                                            inst_ptr = kit_mut.pool_inst_mut.make_global_float(f);
                                            //这里
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
                            InitVal::InitValVec(val_vec) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray((id)) => {
                            if !kit_mut
                                .context_mut
                                .add_var(id.as_str(), Type::Float, false, false,vec![])
                            {
                                return Err(Error::MultipleDeclaration);
                            }
                            if kit_mut.context_mut.get_layer()==-1{//设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                let inst_ptr = kit_mut.pool_inst_mut.make_global_float(0.0);
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
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
    type Message = (Type,Vec<i32>);
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
                
                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                let params_vec = params.process(1, kit_mut).unwrap();
                for (name, param) in params_vec {
                    // kit_mut.add_var(&name, tp, is_array, dimension)
                    func_mut.set_parameter(name, param); //这里
                }
                kit_mut.context_mut.bb_now_set(bb);
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
                        .add_var(id, Type::Int, false, true,Vec::new());
                    //这里
                    kit_mut.context_mut.update_var_scope_now(id, param);
                    Ok((id.clone(), param))
                }
                BType::Float => {
                    let param = kit_mut.pool_inst_mut.make_param(IrType::Float);
                    kit_mut
                        .context_mut
                        .add_var(id, Type::Float, false, true,Vec::new());
                    kit_mut.context_mut.update_var_scope_now(id, param);
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
                exp_stmt.process(Type::Int, kit_mut); //这里可能有问题
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
        // println!("assign stmt");
        let mut mes = Type::Int;
        match symbol.tp {
            Type::ConstFloat => {
                mes = Type::Float;
            }
            Type::ConstInt => {
                mes = Type::Int;
            }
            Type::Float => {
                mes = Type::Float;
            }
            Type::Int => {
                mes = Type::Int;
            }
        }
        
// println!("zhe");
        let (inst_r, _) = self.exp.process(mes, kit_mut).unwrap();
        if symbol.is_param{
            kit_mut.param_used(&lval.id);
        }
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
        Ok(1)
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
            let (inst, _) = exp.process(Type::Int, kit_mut).unwrap(); //这里可能有问题
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
    type Ret = (ObjPtr<Inst>, ExpValue);
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
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // let id = self.id;
        // let vec = self.exp_vec;
        let (var, symbol) = kit_mut.get_var(&self.id).unwrap();
        // println!("var_name:{:?},ir_type:{:?}",&self.id,var.as_ref().get_ir_type());
        // println!("var_name:{:?},ir_type:{:?}",&self.id,var.as_ref().get_kind());
        match input {
            Type::ConstFloat | Type::Float => {
                match symbol.tp {
                    Type::Int | Type::ConstInt => {
                        let inst_trans = kit_mut.pool_inst_mut.make_int_to_float(var);
                        // let mut val = var.as_ref().get_float_bond();
                        // let mut val_ret = ExpValue::Int(val as i32);
                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;
                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((inst_trans, val_ret));
                            }
                            _ => {
                                let mut val_ret = ExpValue::None;
                                // if kit_mut.context_mut.get_layer()<0{
                                //     let val = var.as_ref().get_int_bond();
                                //     val_ret = ExpValue::Float(val as f32);
                                // }
                                
                                match var.as_ref().get_kind() {
                                    InstKind::ConstInt(i)|InstKind::GlobalInt(i)|InstKind::GlobalConstInt(i) =>{
                                        val_ret = ExpValue::Float(i as f32);
                                    }
                                    _=>{}
                                }
                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((inst_trans, val_ret));
                            }
                        }
                    }
                    _ => {
                        // let mut val = var.as_ref().get_float_bond();
                        // let val_ret = ExpValue::Float(val);
                        // return Ok((var,val_ret));

                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;

                                return Ok((var, val_ret));
                            }
                            _ => {
                                // let mut val = var.as_ref().get_float_bond();
                                // let mut val_ret = ExpValue::Float(val);
                                let mut val_ret = ExpValue::None;

                                match var.as_ref().get_kind() {
                                    InstKind::ConstFloat(f)|InstKind::GlobalFloat(f)|InstKind::GlobalConstFloat(f) =>{
                                        val_ret = ExpValue::Float(f);
                                    }
                                    _=>{}
                                }
                                // if kit_mut.context_mut.get_layer()<0{
                                //     let val = var.as_ref().get_float_bond();
                                //     val_ret = ExpValue::Float(val);
                                // }
                                // kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((var, val_ret));
                            }
                        }
                    }
                }
            }
            Type::ConstInt | Type::Int => {
                match symbol.tp {
                    Type::Float | Type::ConstFloat => {
                        let inst_trans = kit_mut.pool_inst_mut.make_float_to_int(var);
                        // let mut val = var.as_ref().get_float_bond();
                        // let mut val_ret = ExpValue::Int(val as i32);
                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;
                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((inst_trans, val_ret));
                            }
                            _ => {
                                let mut val_ret = ExpValue::None;
                                // if kit_mut.context_mut.get_layer()<0{
                                //     let val = var.as_ref().get_float_bond();
                                //     val_ret = ExpValue::Int(val as i32);
                                // }

                                match var.as_ref().get_kind() {
                                    InstKind::ConstFloat(f)|InstKind::GlobalFloat(f)|InstKind::GlobalConstFloat(f) =>{
                                        val_ret = ExpValue::Int(f as i32);
                                    }
                                    _=>{}
                                }
                                
                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((inst_trans, val_ret));
                            }
                        }
                    }
                    _ => {
                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;

                                return Ok((var, val_ret));
                            }
                            _ => {
                                // println!("var:{:?},var_type:{:?}",var.as_ref().get_kind(),var.as_ref().get_ir_type());
                                let mut val_ret = ExpValue::None;
                                match var.as_ref().get_kind() {
                                    InstKind::ConstInt(i)|InstKind::GlobalInt(i)|InstKind::GlobalConstInt(i) =>{
                                        val_ret = ExpValue::Int(i);
                                    }
                                    _=>{}
                                }
                                // if kit_mut.context_mut.get_layer()<0{
                                //     let val = var.as_ref().get_int_bond();
                                //     val_ret = ExpValue::Int(val);
                                // }

                                // let mut val = var.as_ref().get_int_bond();
                                // let mut val_ret = ExpValue::Int(val);
                                // kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((var, val_ret));
                            }
                        }
                    }
                }
            }
        }
        // if symbol.is_array {
        //     todo!();
        // } else {
        //     return Ok(var);
        // }
    }
}

impl Process for PrimaryExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
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
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Number::FloatConst(f) => {
                if let Some(inst) = kit_mut.context_mut.get_const_float(*f) {
                    return Ok((inst, ExpValue::Float(*f)));
                } else {
                    let inst = kit_mut.pool_inst_mut.make_float_const(*f);
                    kit_mut.context_mut.add_const_float(*f, inst);
                    return Ok((inst, ExpValue::Float(*f)));
                }
            }
            Number::IntConst(i) => {
                match input {
                    Type::ConstFloat | Type::Float => {
                        let f = *i as f32;
                        if let Some(inst) = kit_mut.context_mut.get_const_float(f) {
                            return Ok((inst, ExpValue::Float(f)));
                        } else {
                            let inst = kit_mut.pool_inst_mut.make_float_const(f);
                            kit_mut.context_mut.add_const_float(f, inst);
                            // println!("intconst:{}", i);
                            return Ok((inst, ExpValue::Float(f)));
                        }
                    }
                    // Type::Float =>{

                    // }
                    Type::ConstInt | Type::Int => {
                        if let Some(inst) = kit_mut.context_mut.get_const_int(*i) {
                            return Ok((inst, ExpValue::Int(*i)));
                        } else {
                            let inst = kit_mut.pool_inst_mut.make_int_const(*i);
                            kit_mut.context_mut.add_const_int(*i, inst);
                            // println!("intconst:{}", i);
                            return Ok((inst, ExpValue::Int(*i)));
                        }
                    } // Type::Int =>{

                      // }
                }
            }
        }
    }
}

impl Process for OptionFuncRParams {
    type Ret = Vec<ObjPtr<Inst>>;
    type Message = (Vec<Type>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if let Some(rparams) = &mut self.func_fparams {
            Ok(rparams.process(input, kit_mut).unwrap())
        } else {
            Ok(vec![])
        }
    }
}
impl Process for UnaryExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            UnaryExp::PrimaryExp(primaryexp) => primaryexp.process(input, kit_mut),
            UnaryExp::OpUnary((unaryop, unaryexp)) => match unaryop {
                UnaryOp::Add => {
                    let (mut inst_u, mut val) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_pos(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = val;
                    Ok((inst, val_ret))
                }
                UnaryOp::Minus => {
                    let (mut inst_u, mut val) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_neg(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = val;
                    match val {
                        ExpValue::Float(f) => {
                            val_ret = ExpValue::Float(-f);
                        }
                        ExpValue::Int(i) => {
                            val_ret = ExpValue::Int(-i);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    }
                    Ok((inst, val_ret))
                }
                UnaryOp::Exclamation => {
                    let (inst_u, _) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_not(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok((inst, ExpValue::None))
                }
            },
            UnaryExp::FuncCall((funcname, funcparams)) => {
                let inst_func = kit_mut.context_mut.module_mut.get_function(&funcname);
                let fparams = inst_func.as_ref().get_parameter_list();
                let mut fparams_type_vec = vec![];
                for fp in fparams {//获得各参数类型
                    match fp.as_ref().get_ir_type() {
                        IrType::Float => {
                            fparams_type_vec.push(Type::Float);
                        }
                        IrType::Int => {
                            fparams_type_vec.push(Type::Int);
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                match inst_func.as_ref().get_return_type() {//根据返回值类型生成call指令
                    IrType::Float => {
                        let mut args = funcparams.process(fparams_type_vec, kit_mut).unwrap();//获得实参
                        let mut fname = " ".to_string();
                        if let Some((funcname_in, _)) = kit_mut
                            .context_mut
                            .module_mut
                            .function
                            .get_key_value(funcname)
                        {
                            fname = funcname_in.clone();
                        }
                        let inst = kit_mut.pool_inst_mut.make_float_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    IrType::Int => {
                        let mut args = funcparams.process(fparams_type_vec, kit_mut).unwrap();
                        let mut fname = " ".to_string();
                        if let Some((funcname_in, _)) = kit_mut
                            .context_mut
                            .module_mut
                            .function
                            .get_key_value(funcname)
                        {
                            fname = funcname_in.clone();
                        }
                        let inst = kit_mut.pool_inst_mut.make_int_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    IrType::Void => {
                        let mut args = funcparams.process(fparams_type_vec, kit_mut).unwrap();
                        let mut fname = " ".to_string();
                        if let Some((funcname_in, _)) = kit_mut
                            .context_mut
                            .module_mut
                            .function
                            .get_key_value(funcname)
                        {
                            fname = funcname_in.clone();
                        }
                        let inst = kit_mut.pool_inst_mut.make_void_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
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
    type Ret = Vec<ObjPtr<Inst>>;
    type Message = (Vec<Type>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // match self{
        //     FuncFParams
        // }
        let mut vec = vec![];
        let mut index = 0;
        for i in &mut self.exp_vec {
            let (inst, _) = i.process(input[index], kit_mut).unwrap();
            vec.push(inst);
            index = index + 1;
        }
        Ok(vec)
    }
}

impl Process for MulExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            MulExp::UnaryExp(unaryexp) => unaryexp.process(input, kit_mut),
            MulExp::MulExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_mul(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 * f2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 * i2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        val_ret = ExpValue::None;
                    }
                }
                Ok((inst, val_ret))
            }
            MulExp::DivExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_div(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 / f2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 / i2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        val_ret = ExpValue::None;
                    }
                }
                Ok((inst, val_ret))
            }
            MulExp::ModExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_rem(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 % f2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 % i2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        val_ret = ExpValue::None;
                    }
                }
                Ok((inst, val_ret))
            }
        }
    }
}
// impl Process for AddOp {
//     type Ret = i32;
//     type Message = (i32);
//     fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
//         todo!();
//     }
// }

impl Process for AddExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            AddExp::MulExp(mulexp) => mulexp.as_mut().process(input, kit_mut),
            AddExp::OpExp((opexp, op, mulexp)) => match op {
                AddOp::Add => {
                    let (inst_left, lval) = opexp.process(input, kit_mut).unwrap();
                    let (inst_right, rval) = mulexp.process(input, kit_mut).unwrap();
                    // println!("lvar:{:?},type:{:?},rvar:{:?},type:{:?}",inst_left.as_ref().get_kind(),inst_left.as_ref().get_ir_type(),inst_right.as_ref().get_kind(),inst_right.as_ref().get_ir_type());
                    let inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = lval;
                    match lval {
                        ExpValue::Float(f1) => match rval {
                            ExpValue::Float(f2) => {
                                val_ret = ExpValue::Float(f1 + f2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        ExpValue::Int(i1) => match rval {
                            ExpValue::Int(i2) => {
                                val_ret = ExpValue::Int(i1 + i2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    }
                    Ok((inst, val_ret))
                }
                AddOp::Minus => {
                    let (inst_left, lval) = opexp.process(input, kit_mut).unwrap();
                    let (inst_right, rval) = mulexp.process(input, kit_mut).unwrap();
                    // let inst_right_neg = kit_mut.pool_inst_mut.make_neg(inst_right);
                    let inst = kit_mut.pool_inst_mut.make_sub(inst_left, inst_right);
                    // kit_mut.context_mut.push_inst_bb(inst_right);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = lval;
                    match lval {
                        ExpValue::Float(f1) => match rval {
                            ExpValue::Float(f2) => {
                                val_ret = ExpValue::Float(f1 - f2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        ExpValue::Int(i1) => match rval {
                            ExpValue::Int(i2) => {
                                val_ret = ExpValue::Int(i1 - i2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    }
                    Ok((inst, val_ret))
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
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.add_exp.process(input, kit_mut)
    }
}

impl Process for LOrExp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}
