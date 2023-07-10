use super::context::Type;
use super::kit::Kit;
use super::{ast::*, context::Context};
use super::{init_padding_float, init_padding_int, ExpValue, InfuncChoice, RetInitVec};
use crate::frontend::context::Symbol;
use crate::frontend::error::Error;
use crate::frontend::typesearch::TypeProcess;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::{Inst, InstKind, UnOp};
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
use crate::utility::{ObjPool, ObjPtr};

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
    kit_mut.init_external_funcs();
    compunit.process(1, &mut kit_mut).unwrap();
    kit_mut.phi_padding_allfunctions();
    kit_mut.merge_allfunctions();
}

pub trait Process {
    type Ret;
    type Message;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error>;
}

impl Process for CompUnit {
    type Ret = i32;
    type Message = i32;
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        for item in &mut self.global_items {
            kit_mut.context_mut.set_stop_genir(false);
            item.process(1, kit_mut).unwrap();
        }
        return Ok(1);
    }
}

impl Process for GlobalItems {
    type Ret = i32;
    type Message = i32;
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if kit_mut.context_mut.stop_genir {
            Ok(1)
        } else {
            match self {
                Self::Decl(decl) => {
                    decl.process(1, kit_mut).unwrap();
                    Ok(1)
                }
                Self::FuncDef(funcdef) => {
                    let inst_func = funcdef.process(true, kit_mut).unwrap();
                    match kit_mut.context_mut.bb_now_mut {
                        InfuncChoice::InFunc(bb_now) => {
                            if !bb_now.get_up_bb().is_empty()
                                || (bb_now.get_up_bb().is_empty() && inst_func.get_head() == bb_now)
                            {
                                if bb_now.is_empty() {
                                    match inst_func.get_return_type() {
                                        IrType::Void => {
                                            bb_now.as_mut().push_back(
                                                kit_mut.pool_inst_mut.make_return_void(),
                                            );
                                            let func_name = &kit_mut.context_mut.func_now;
                                            if let Some(vec) = kit_mut
                                                .context_mut
                                                .terminated_map
                                                .get_mut(func_name)
                                            {
                                                vec.push((
                                                    bb_now,
                                                    kit_mut.pool_inst_mut.make_int_const(-1129),
                                                ));
                                            }
                                        }
                                        IrType::Int => {
                                            let inst_val = kit_mut.pool_inst_mut.make_int_const(0);
                                            bb_now.as_mut().push_back(inst_val);
                                            bb_now.as_mut().push_back(
                                                kit_mut.pool_inst_mut.make_return(inst_val),
                                            );
                                            let func_name = &kit_mut.context_mut.func_now;
                                            if let Some(vec) = kit_mut
                                                .context_mut
                                                .terminated_map
                                                .get_mut(func_name)
                                            {
                                                vec.push((bb_now, inst_val));
                                            }
                                        }
                                        IrType::Float => {
                                            let inst_val =
                                                kit_mut.pool_inst_mut.make_float_const(0.0);
                                            bb_now.as_mut().push_back(inst_val);
                                            bb_now.as_mut().push_back(
                                                kit_mut.pool_inst_mut.make_return(inst_val),
                                            );
                                            let func_name = &kit_mut.context_mut.func_now;
                                            if let Some(vec) = kit_mut
                                                .context_mut
                                                .terminated_map
                                                .get_mut(func_name)
                                            {
                                                vec.push((bb_now, inst_val));
                                            }
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                } else {
                                    match bb_now.get_tail_inst().get_kind() {
                                        InstKind::Return => {}
                                        _ => match inst_func.get_return_type() {
                                            IrType::Void => {
                                                bb_now.as_mut().push_back(
                                                    kit_mut.pool_inst_mut.make_return_void(),
                                                );
                                                let func_name = &kit_mut.context_mut.func_now;
                                                if let Some(vec) = kit_mut
                                                    .context_mut
                                                    .terminated_map
                                                    .get_mut(func_name)
                                                {
                                                    vec.push((
                                                        bb_now,
                                                        kit_mut.pool_inst_mut.make_int_const(-1129),
                                                    ));
                                                }
                                            }
                                            IrType::Int => {
                                                let inst_val =
                                                    kit_mut.pool_inst_mut.make_int_const(0);
                                                bb_now.as_mut().push_back(inst_val);
                                                bb_now.as_mut().push_back(
                                                    kit_mut.pool_inst_mut.make_return(inst_val),
                                                );
                                                let func_name = &kit_mut.context_mut.func_now;
                                                if let Some(vec) = kit_mut
                                                    .context_mut
                                                    .terminated_map
                                                    .get_mut(func_name)
                                                {
                                                    vec.push((bb_now, inst_val));
                                                }
                                            }
                                            IrType::Float => {
                                                let inst_val =
                                                    kit_mut.pool_inst_mut.make_float_const(0.0);
                                                bb_now.as_mut().push_back(inst_val);
                                                bb_now.as_mut().push_back(
                                                    kit_mut.pool_inst_mut.make_return(inst_val),
                                                );
                                                let func_name = &kit_mut.context_mut.func_now;
                                                if let Some(vec) = kit_mut
                                                    .context_mut
                                                    .terminated_map
                                                    .get_mut(func_name)
                                                {
                                                    vec.push((bb_now, inst_val));
                                                }
                                            }
                                            _ => {
                                                unreachable!(
                                                "返回值非空,但最后一个块的最后一条指令却不是ret"
                                            )
                                            }
                                        },
                                    }
                                }
                            } //
                        }
                        _ => {
                            unreachable!("函数ir生成完毕,本应处于最后一个bb中")
                        }
                    }
                    kit_mut.context_mut.bb_now_mut = InfuncChoice::NInFunc();
                    kit_mut.context_mut.set_stop_genir(false);
                    Ok(1)
                }
            }
        }
    }
}

impl Process for Decl {
    type Ret = i32;
    type Message = i32;

    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if kit_mut.context_mut.stop_genir {
            Ok(1)
        } else {
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
        }
    }
}

impl Process for ConstDecl {
    type Ret = i32;
    type Message = i32;
    fn process(&mut self, _: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self.btype {
            BType::Int => {
                for def in &mut self.const_def_vec {
                    if def.const_exp_vec.is_empty() {
                        //非数组
                        let (mut inst_ptr, val, _) = def
                            .const_init_val
                            .process((Type::ConstInt, vec![], 0, 1), kit_mut)
                            .unwrap();

                        match val {
                            //构造const指令
                            ExpValue::Int(i) => {
                                if kit_mut.context_mut.get_layer() < 0 {
                                    inst_ptr = kit_mut.pool_inst_mut.make_global_int_const(i);
                                } else {
                                    inst_ptr = kit_mut.pool_inst_mut.make_int_const(i);
                                    kit_mut.context_mut.push_inst_bb(inst_ptr); //update会将全局变量放入module中不会将局部变量放入bb中
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                        if kit_mut.context_mut.get_layer() < 0 {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstInt,
                                false,
                                false,
                                None,
                                Some(inst_ptr),
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        } else {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstInt,
                                false,
                                false,
                                None,
                                None,
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        }
                        //这里

                        kit_mut
                            .context_mut
                            .update_var_scope_now(&def.ident, inst_ptr); //update会将全局变量放入module中不会将局部变量放入bb中
                    } else {
                        //数组
                        let mut dimension = vec![];
                        for exp in &mut def.const_exp_vec {
                            dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                        let dimension_vec_in: Vec<_> = dimension_vec
                            .iter()
                            .map(|x| match x {
                                ExpValue::Int(i) => *i,
                                ExpValue::Float(f) => {
                                    unreachable!()
                                }
                                ExpValue::None => {
                                    unreachable!()
                                }
                                _ => {
                                    unreachable!()
                                }
                            })
                            .collect(); //生成维度vec

                        let mut length = 1;
                        for dm in &dimension_vec_in {
                            length = length * dm;
                        }

                        let (_, _, init_vec) = def
                            .const_init_val
                            .process((Type::ConstInt, dimension_vec_in.clone(), 0, 1), kit_mut)
                            .unwrap(); //获得初始值
                        match init_vec {
                            RetInitVec::Float(_) => {
                                unreachable!()
                            }
                            RetInitVec::Int(ivec) => {
                                let mut vec_init = vec![];
                                for i in ivec {
                                    vec_init.push((false, i));
                                }
                                let mut inst =
                                    kit_mut.pool_inst_mut.make_int_array(length, true, vec_init);
                                match &def.const_init_val {
                                    ConstInitVal::ConstInitValVec(vec_temp) => {
                                        if vec_temp.is_empty() {
                                            inst = kit_mut.pool_inst_mut.make_int_array(
                                                length,
                                                true,
                                                vec![],
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                                if !kit_mut.context_mut.add_var(
                                    &def.ident,
                                    Type::ConstInt,
                                    true,
                                    false,
                                    Some(inst),
                                    None,
                                    dimension_vec_in.clone(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                } //添加该变量，但没有生成实际的指令
                                kit_mut.context_mut.update_var_scope_now(&def.ident, inst);
                                kit_mut.context_mut.push_inst_bb(inst);
                            }
                        }
                    }
                }
                return Ok(1);
            }
            BType::Float => {
                for def in &mut self.const_def_vec {
                    if def.const_exp_vec.is_empty() {
                        let (mut inst_ptr, val, _) = def
                            .const_init_val
                            .process((Type::ConstFloat, vec![], 0, 1), kit_mut)
                            .unwrap();

                        let mut bond = 0.0;
                        match val {
                            ExpValue::Float(i) => {
                                bond = i;
                                if kit_mut.context_mut.get_layer() < 0 {
                                    inst_ptr = kit_mut.pool_inst_mut.make_global_float_const(bond);
                                } else {
                                    inst_ptr = kit_mut.pool_inst_mut.make_float_const(i);
                                    kit_mut.context_mut.push_inst_bb(inst_ptr);
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }

                        if kit_mut.context_mut.get_layer() < 0 {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstFloat,
                                false,
                                false,
                                None,
                                Some(inst_ptr),
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        } else {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstFloat,
                                false,
                                false,
                                None,
                                None,
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        }
                        //这里
                        kit_mut
                            .context_mut
                            .update_var_scope_now(&def.ident, inst_ptr);
                    } else {
                        //数组
                        let mut dimension = vec![];
                        for exp in &mut def.const_exp_vec {
                            dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                        let dimension_vec_in: Vec<_> = dimension_vec
                            .iter()
                            .map(|x| match x {
                                ExpValue::Int(i) => *i,
                                ExpValue::Float(_) => {
                                    unreachable!()
                                }
                                ExpValue::None => {
                                    unreachable!()
                                }
                                _ => {
                                    unreachable!()
                                }
                            })
                            .collect(); //生成维度vec

                        let mut length = 1;
                        for dm in &dimension_vec_in {
                            length = length * dm;
                        }

                        let (_, _, init_vec) = def
                            .const_init_val
                            .process((Type::ConstFloat, dimension_vec_in.clone(), 0, 1), kit_mut)
                            .unwrap(); //获得初始值
                        match init_vec {
                            RetInitVec::Float(fvec) => {
                                let mut vec_init = vec![];
                                for f in fvec {
                                    vec_init.push((false, f));
                                }
                                let mut inst = kit_mut
                                    .pool_inst_mut
                                    .make_float_array(length, true, vec_init);
                                match &def.const_init_val {
                                    //这里,加个判断，为空则放个空的
                                    ConstInitVal::ConstInitValVec(vec_temp) => {
                                        if vec_temp.is_empty() {
                                            inst = kit_mut.pool_inst_mut.make_float_array(
                                                length,
                                                true,
                                                vec![],
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                                if !kit_mut.context_mut.add_var(
                                    &def.ident,
                                    Type::ConstInt,
                                    true,
                                    false,
                                    Some(inst),
                                    None,
                                    dimension_vec_in.clone(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                } //添加该变量，但没有生成实际的指令
                                kit_mut.context_mut.update_var_scope_now(&def.ident, inst);
                                kit_mut.context_mut.push_inst_bb(inst);
                            }
                            RetInitVec::Int(_) => {
                                unreachable!()
                            }
                        }
                    }
                }
                return Ok(1);
            }
        }
    }
}

impl Process for ConstInitVal {
    type Ret = (ObjPtr<Inst>, ExpValue, RetInitVec);
    type Message = (Type, Vec<i32>, i32, i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            ConstInitVal::ConstExp(constexp) => {
                let (inst, value) = constexp.process(input.0, kit_mut).unwrap();
                match value {
                    ExpValue::Float(f) => {
                        let mut vec_ret = Vec::new();
                        vec_ret.push(f);
                        Ok((inst, value, RetInitVec::Float(vec_ret)))
                    }
                    ExpValue::Int(i) => {
                        let mut vec_ret = Vec::new();
                        vec_ret.push(i);
                        Ok((inst, value, RetInitVec::Int(vec_ret)))
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            ConstInitVal::ConstInitValVec(constvalvec) => {
                let (tp, dimension_vec, mut num_pre, layer_now) = input;
                let num_pre_tmp = num_pre;
                let mut vec_ret_float = vec![];
                let mut vec_ret_int = vec![];
                let mut total = 1;
                let mut vec_ttt = vec![];
                for i in (layer_now as usize - 1)..dimension_vec.len() {
                    vec_ttt.push(dimension_vec[i]);
                }
                for i in 0..dimension_vec.len() {
                    total = total * dimension_vec[i];
                }
                // let after = dimension_vec[dimension_vec.len() - 1];
                for val in constvalvec {
                    let (_, _, vec_temp) = val
                        .process((tp, dimension_vec.clone(), num_pre, layer_now + 1), kit_mut)
                        .unwrap(); //子init_vec生成vec
                    match vec_temp {
                        //将子vec中值放到当前vec中
                        RetInitVec::Float(vec_float) => match tp {
                            Type::Float | Type::ConstFloat => {
                                num_pre = num_pre + vec_float.len() as i32;
                                for val_son in vec_float {
                                    vec_ret_float.push(val_son);
                                }
                            }
                            _ => {
                                unreachable!();
                            }
                        },
                        RetInitVec::Int(vec_int) => match tp {
                            Type::Int | Type::ConstInt => {
                                num_pre = num_pre + vec_int.len() as i32;
                                for val_son in vec_int {
                                    vec_ret_int.push(val_son);
                                }
                            }
                            _ => {
                                unreachable!();
                            }
                        },
                    }
                }

                match tp {
                    Type::Float | Type::ConstFloat => {
                        init_padding_float(&mut vec_ret_float, vec_ttt, num_pre_tmp, total);
                        return Ok((
                            kit_mut.pool_inst_mut.make_int_const(-1),
                            ExpValue::None,
                            RetInitVec::Float(vec_ret_float),
                        ));
                    }
                    Type::Int | Type::ConstInt => {
                        init_padding_int(&mut vec_ret_int, vec_ttt, num_pre_tmp, total);
                        return Ok((
                            kit_mut.pool_inst_mut.make_int_const(-1),
                            ExpValue::None,
                            RetInitVec::Int(vec_ret_int),
                        ));
                    }
                    _ => {
                        todo!()
                    }
                }
            }
        }
    }
}

impl Process for VarDecl {
    type Ret = i32;
    type Message = i32;
    fn process(&mut self, _: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self.btype {
            BType::Int => {
                for def in &mut self.var_def_vec {
                    match def {
                        VarDef::NonArrayInit((id, val)) => match val {
                            InitVal::Exp(exp) => {
                                let flag = exp.type_process(1, kit_mut).unwrap();
                                let mut tp = Type::Int;
                                if flag > 1 {
                                    tp = Type::Float;
                                }
                                let (mut inst_ptr, val) = exp.process(tp, kit_mut).unwrap();
                                let mut flag_trans = false;
                                if flag > 1 {
                                    flag_trans = true;
                                    // inst_ptr = kit_mut.pool_inst_mut.make_float_to_int(inst_ptr);
                                    // kit_mut.context_mut.push_inst_bb(inst_ptr); //先放进去,不管是否冗余
                                }

                                if kit_mut.context_mut.get_layer() < 0 {
                                    //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                    match val {
                                        ExpValue::Int(i) => {
                                            inst_ptr = kit_mut.pool_inst_mut.make_global_int(i);
                                            if !kit_mut.context_mut.add_var(
                                                id,
                                                Type::Int,
                                                false,
                                                false,
                                                None,
                                                Some(inst_ptr),
                                                Vec::new(),
                                            ) {
                                                return Err(Error::MultipleDeclaration);
                                            }
                                            //这里
                                        }
                                        ExpValue::Float(f) => {
                                            inst_ptr =
                                                kit_mut.pool_inst_mut.make_global_int(f as i32);
                                            if !kit_mut.context_mut.add_var(
                                                id,
                                                Type::Int,
                                                false,
                                                false,
                                                None,
                                                Some(inst_ptr),
                                                Vec::new(),
                                            ) {
                                                return Err(Error::MultipleDeclaration);
                                            }
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                } else {
                                    if flag_trans {
                                        match val {
                                            ExpValue::Float(f) => {
                                                inst_ptr =
                                                    kit_mut.pool_inst_mut.make_int_const(f as i32);
                                            }
                                            _ => {
                                                inst_ptr = kit_mut
                                                    .pool_inst_mut
                                                    .make_float_to_int(inst_ptr);
                                            }
                                        }
                                        kit_mut.context_mut.push_inst_bb(inst_ptr);
                                    }

                                    if !kit_mut.context_mut.add_var(
                                        id,
                                        Type::Int,
                                        false,
                                        false,
                                        None,
                                        None,
                                        Vec::new(),
                                    ) {
                                        return Err(Error::MultipleDeclaration);
                                    }
                                }

                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
                            InitVal::InitValVec(_) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray(id) => {
                            if kit_mut.context_mut.get_layer() == -1 {
                                //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                let inst_ptr = kit_mut.pool_inst_mut.make_global_int(0);
                                if !kit_mut.context_mut.add_var(
                                    id,
                                    Type::Int,
                                    false,
                                    false,
                                    None,
                                    Some(inst_ptr),
                                    Vec::new(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            } else {
                                if !kit_mut.context_mut.add_var(
                                    id,
                                    Type::Int,
                                    false,
                                    false,
                                    None,
                                    None,
                                    Vec::new(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                let inst_ptr = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                                kit_mut.context_mut.push_inst_bb(inst_ptr);
                            }
                        }
                        VarDef::ArrayInit((id, exp_vec, val)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(_) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec

                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }
                            let (init_vec, inst_vec) = val
                                .process((Type::Int, dimension_vec_in.clone(), 0, 1), kit_mut)
                                .unwrap(); //获得初始值
                            match init_vec {
                                RetInitVec::Int(mut ivec) => {
                                    let mut inst = kit_mut.pool_inst_mut.make_int_const(-1129);
                                    // let inst = kit_mut.pool_inst_mut.make_int_array(length, ivec);
                                    let mut last_index = 0;
                                    let mut vec_init = vec![];
                                    for i in 0..ivec.len() {
                                        vec_init.push((false, ivec[i]));
                                        if ivec[i] != 0 {
                                            last_index = i + 1;
                                        }
                                    }
                                    for option_exp in &inst_vec {
                                        if let Some((_, offset_val)) = option_exp {
                                            if last_index < *offset_val as usize + 1 {
                                                last_index = *offset_val as usize + 1;
                                            }
                                            vec_init[*offset_val as usize] = (true, 0);
                                        }
                                    }
                                    if last_index > 0 {
                                        for i in (last_index - 1)..ivec.len() {
                                            vec_init.remove(i);
                                        }
                                        inst = kit_mut
                                            .pool_inst_mut
                                            .make_int_array(length, true, vec_init);
                                    } else {
                                        inst = kit_mut.pool_inst_mut.make_int_array(
                                            length,
                                            true,
                                            vec![],
                                        );
                                    }
                                    if !kit_mut.context_mut.add_var(
                                        &id,
                                        Type::Int,
                                        true,
                                        false,
                                        Some(inst),
                                        None, //可能得改,数组的就没有对全局区域留inst
                                        dimension_vec_in.clone(),
                                    ) {
                                        return Err(Error::MultipleDeclaration);
                                    } //添加该变量，但没有生成实际的指令
                                    kit_mut.context_mut.update_var_scope_now(&id, inst);
                                    kit_mut.context_mut.push_inst_bb(inst);
                                    for option_exp in inst_vec {
                                        if let Some((inst_val, offset_val)) = option_exp {
                                            let offset =
                                                kit_mut.pool_inst_mut.make_int_const(offset_val);
                                            let ptr = kit_mut.pool_inst_mut.make_gep(inst, offset);
                                            let inst_store =
                                                kit_mut.pool_inst_mut.make_int_store(ptr, inst_val);
                                            kit_mut.context_mut.push_inst_bb(offset);
                                            kit_mut.context_mut.push_inst_bb(ptr);
                                            kit_mut.context_mut.push_inst_bb(inst_store);
                                        }
                                    }
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        VarDef::Array((id, exp_vec)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(_) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec

                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }
                            let ivec = vec![];
                            let inst = kit_mut.pool_inst_mut.make_int_array(length, false, ivec);
                            if !kit_mut.context_mut.add_var(
                                &id,
                                Type::Int,
                                true,
                                false,
                                Some(inst),
                                None,
                                dimension_vec_in.clone(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            } //添加该变量，但没有生成实际的指令
                            kit_mut.context_mut.update_var_scope_now(&id, inst);
                            kit_mut.context_mut.push_inst_bb(inst);
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
                                let flag = exp.type_process(1, kit_mut).unwrap();
                                let mut tp = Type::Int;
                                if flag > 1 {
                                    tp = Type::Float;
                                }
                                let (mut inst_ptr, val) = exp.process(tp, kit_mut).unwrap();
                                let mut flag_trans = false;
                                if flag <= 1 {
                                    flag_trans = true;
                                }
                                if kit_mut.context_mut.get_layer() < 0 {
                                    //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                    match val {
                                        ExpValue::Float(f) => {
                                            inst_ptr = kit_mut.pool_inst_mut.make_global_float(f);
                                            if !kit_mut.context_mut.add_var(
                                                id,
                                                Type::Float,
                                                false,
                                                false,
                                                None,
                                                Some(inst_ptr),
                                                Vec::new(),
                                            ) {
                                                return Err(Error::MultipleDeclaration);
                                            }
                                            //这里
                                        }
                                        ExpValue::Int(i) => {
                                            inst_ptr =
                                                kit_mut.pool_inst_mut.make_global_float(i as f32);
                                            if !kit_mut.context_mut.add_var(
                                                id,
                                                Type::Float,
                                                false,
                                                false,
                                                None,
                                                Some(inst_ptr),
                                                Vec::new(),
                                            ) {
                                                return Err(Error::MultipleDeclaration);
                                            }
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                } else {
                                    if flag_trans {
                                        match val {
                                            ExpValue::Int(i) => {
                                                inst_ptr = kit_mut
                                                    .pool_inst_mut
                                                    .make_float_const(i as f32);
                                            }
                                            _ => {
                                                inst_ptr = kit_mut
                                                    .pool_inst_mut
                                                    .make_int_to_float(inst_ptr);
                                            }
                                        }
                                        kit_mut.context_mut.push_inst_bb(inst_ptr);
                                    }
                                    if !kit_mut.context_mut.add_var(
                                        id,
                                        Type::Float,
                                        false,
                                        false,
                                        None,
                                        None,
                                        Vec::new(),
                                    ) {
                                        return Err(Error::MultipleDeclaration);
                                    }
                                }

                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
                            InitVal::InitValVec(_) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray(id) => {
                            if kit_mut.context_mut.get_layer() == -1 {
                                //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                let inst_ptr = kit_mut.pool_inst_mut.make_global_float(0.0);
                                if !kit_mut.context_mut.add_var(
                                    id.as_str(),
                                    Type::Float,
                                    false,
                                    false,
                                    None,
                                    Some(inst_ptr),
                                    vec![],
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            } else {
                                if !kit_mut.context_mut.add_var(
                                    id.as_str(),
                                    Type::Float,
                                    false,
                                    false,
                                    None,
                                    None,
                                    vec![],
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                let inst_ptr = kit_mut.pool_inst_mut.make_float_const(0.0);
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                                kit_mut.context_mut.push_inst_bb(inst_ptr);
                            }
                        }
                        VarDef::ArrayInit((id, exp_vec, val)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(_) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec
                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }
                            let (init_vec, inst_vec) = val
                                .process((Type::Float, dimension_vec_in.clone(), 0, 1), kit_mut)
                                .unwrap(); //获得初始值
                            match init_vec {
                                RetInitVec::Float(mut fvec) => {
                                    let mut last_index = 0;
                                    let mut vec_init = vec![];
                                    for i in 0..fvec.len() {
                                        vec_init.push((false, fvec[i]));
                                        if fvec[i] != 0.0 {
                                            last_index = i + 1;
                                        }
                                    }
                                    for option_exp in &inst_vec {
                                        if let Some((_, offset_val)) = option_exp {
                                            if last_index < *offset_val as usize + 1 {
                                                last_index = *offset_val as usize + 1;
                                            }
                                            vec_init[*offset_val as usize] = (true, 0.0);
                                        }
                                    }

                                    let mut inst = kit_mut.pool_inst_mut.make_float_const(-1129.0);
                                    if last_index > 0 {
                                        for i in last_index - 1..fvec.len() {
                                            vec_init.remove(i);
                                        }
                                        inst = kit_mut
                                            .pool_inst_mut
                                            .make_float_array(length, true, vec_init);
                                    } else {
                                        inst = kit_mut.pool_inst_mut.make_float_array(
                                            length,
                                            true,
                                            vec![],
                                        );
                                    }
                                    // let inst = kit_mut.pool_inst_mut.make_float_array(length, fvec);
                                    if !kit_mut.context_mut.add_var(
                                        &id,
                                        Type::Float,
                                        true,
                                        false,
                                        Some(inst),
                                        None,
                                        dimension_vec_in.clone(),
                                    ) {
                                        return Err(Error::MultipleDeclaration);
                                    } //添加该变量，但没有生成实际的指令
                                    kit_mut.context_mut.update_var_scope_now(&id, inst);
                                    kit_mut.context_mut.push_inst_bb(inst);
                                    for option_exp in inst_vec {
                                        if let Some((inst_val, offset_val)) = option_exp {
                                            let offset =
                                                kit_mut.pool_inst_mut.make_int_const(offset_val);
                                            let ptr = kit_mut.pool_inst_mut.make_gep(inst, offset);
                                            let inst_store = kit_mut
                                                .pool_inst_mut
                                                .make_float_store(ptr, inst_val);
                                            kit_mut.context_mut.push_inst_bb(offset);
                                            kit_mut.context_mut.push_inst_bb(ptr);
                                            kit_mut.context_mut.push_inst_bb(inst_store);
                                        }
                                    }
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        VarDef::Array((id, exp_vec)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(_) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec

                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }
                            let fvec = vec![];
                            let inst = kit_mut.pool_inst_mut.make_float_array(length, true, fvec);
                            if !kit_mut.context_mut.add_var(
                                &id,
                                Type::Float,
                                true,
                                false,
                                Some(inst),
                                None,
                                dimension_vec_in.clone(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            } //添加该变量，但没有生成实际的指令
                            kit_mut.context_mut.update_var_scope_now(&id, inst);
                            kit_mut.context_mut.push_inst_bb(inst);
                        }
                    }
                }
                Ok(1)
            }
        }
    }
}

impl Process for InitVal {
    type Ret = (RetInitVec, Vec<Option<(ObjPtr<Inst>, i32)>>);
    type Message = (Type, Vec<i32>, i32, usize);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            InitVal::Exp(exp) => {
                let (tp, _, num_precessor, _) = input;
                let (inst, val) = exp.process(tp, kit_mut).unwrap();
                let mut vecf = vec![];
                let mut veci = vec![];
                let mut inst_vec = vec![];
                match val {
                    ExpValue::Float(f) => match tp {
                        Type::Float | Type::ConstFloat => {
                            vecf.push(f);
                            inst_vec.push(None);
                        }
                        _ => {
                            unreachable!()
                        }
                    },
                    ExpValue::Int(i) => match tp {
                        Type::Int | Type::ConstInt => {
                            veci.push(i);
                            inst_vec.push(None);
                        }
                        _ => {
                            unreachable!()
                        }
                    },
                    ExpValue::None => match tp {
                        Type::Float | Type::ConstFloat => {
                            vecf.push(0.0);
                            inst_vec.push(Some((inst, num_precessor)));
                        }
                        Type::Int | Type::ConstInt => {
                            veci.push(0);
                            inst_vec.push(Some((inst, num_precessor)));
                        }
                        _ => {
                            unreachable!()
                        }
                    },
                    _ => {
                        unreachable!()
                    }
                }
                match tp {
                    Type::Float | Type::ConstFloat => Ok((RetInitVec::Float(vecf), inst_vec)),
                    Type::Int | Type::ConstInt => Ok((RetInitVec::Int(veci), inst_vec)),
                    _ => {
                        todo!()
                    }
                }
            }

            InitVal::InitValVec(initvec) => {
                let (tp, dimension, num_precessor, layer_now) = input;
                let mut vec_val_f = vec![];
                let mut vec_val_i = vec![];
                let mut vec_inst_init = vec![];
                let mut after = 1;
                for i in layer_now..dimension.len() {
                    after = after * dimension[i];
                } //计算当前维度每增1对应多少元素
                  ////这里:!
                  // if layer_now==1{\

                // after = dimension[dimension.len() - 1];/////这

                //   }

                let mut vec_dimension_now = vec![];
                for i in (layer_now - 1)..dimension.len() {
                    vec_dimension_now.push(dimension[i]);
                } //计算当前维度每增1对应多少元素

                let mut index = 0; //当前相对位移
                for init in initvec {
                    match init {
                        InitVal::Exp(_) => {
                            let (vec_val_temp, vec_inst_temp) = init
                                .process(
                                    (tp, dimension.clone(), num_precessor + index, layer_now),
                                    kit_mut,
                                )
                                .unwrap();
                            match vec_val_temp {
                                RetInitVec::Float(vec_f) => {
                                    for val in vec_f {
                                        vec_val_f.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(_) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                                RetInitVec::Int(vec_i) => {
                                    for val in vec_i {
                                        vec_val_i.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(_) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                            }

                            index = index + 1; //init为exp，相对偏移加1
                        }
                        InitVal::InitValVec(_) => {
                            let (vec_val_temp, vec_inst_temp) = init
                                .process(
                                    (tp, dimension.clone(), num_precessor + index, layer_now + 1),
                                    kit_mut,
                                )
                                .unwrap();
                            match vec_val_temp {
                                RetInitVec::Float(vec_f) => {
                                    index = index + vec_f.len() as i32;
                                    for val in vec_f {
                                        vec_val_f.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(_) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                                RetInitVec::Int(vec_i) => {
                                    index = index + vec_i.len() as i32;
                                    for val in vec_i {
                                        vec_val_i.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(_) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                            }
                            // index = index + after; //init为vec,相对偏移加after
                        }
                    }
                }
                let mut ttotal = 1;
                for i in 0..dimension.len() {
                    ttotal = ttotal * dimension[i];
                }
                match tp {
                    Type::Float | Type::ConstFloat => {
                        init_padding_float(
                            &mut vec_val_f,
                            vec_dimension_now,
                            num_precessor,
                            ttotal,
                        );
                        Ok((RetInitVec::Float(vec_val_f), vec_inst_init))
                    }
                    Type::Int | Type::ConstInt => {
                        init_padding_int(&mut vec_val_i, vec_dimension_now, num_precessor, ttotal);
                        Ok((RetInitVec::Int(vec_val_i), vec_inst_init))
                    }
                    _ => {
                        todo!()
                    }
                }
            }
        }
    }
}
impl Process for FuncDef {
    type Ret = ObjPtr<Function>;
    type Message = bool;
    fn process(&mut self, _: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::NonParameterFuncDef((tp, id, blk)) => {
                kit_mut.context_mut.set_funcnow(id.to_string());
                let vec_ttt = vec![];
                kit_mut
                    .context_mut
                    .terminated_map
                    .insert(id.to_string(), vec_ttt);

                kit_mut.context_mut.add_layer();
                let func_ptr = kit_mut.pool_func_mut.new_function();
                let func_mut = func_ptr.as_mut();
                let bb = kit_mut.pool_bb_mut.new_basic_block(id.clone());
                func_mut.insert_first_bb(bb);
                match tp {
                    FuncType::Void => {
                        func_mut.set_return_type(IrType::Void);
                        kit_mut.context_mut.set_return_type(Type::NotForce);
                    }
                    FuncType::Int => {
                        func_mut.set_return_type(IrType::Int);
                        kit_mut.context_mut.set_return_type(Type::Int);
                    }
                    FuncType::Float => {
                        func_mut.set_return_type(IrType::Float);
                        kit_mut.context_mut.set_return_type(Type::Float);
                    }
                }
                kit_mut.context_mut.bb_now_set(bb);
                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                blk.process((None, None), kit_mut).unwrap();
                kit_mut.context_mut.delete_layer();
                return Ok(func_ptr);
            }
            Self::ParameterFuncDef((tp, id, params, blk)) => {
                kit_mut.context_mut.set_funcnow(id.to_string());
                let vec_ttt = vec![];
                kit_mut
                    .context_mut
                    .terminated_map
                    .insert(id.to_string(), vec_ttt);

                kit_mut.context_mut.add_layer();
                let func_ptr = kit_mut.pool_func_mut.new_function();
                let func_mut = func_ptr.as_mut();
                let bb = kit_mut.pool_bb_mut.new_basic_block(id.clone());
                func_mut.insert_first_bb(bb);
                match tp {
                    FuncType::Void => {
                        func_mut.set_return_type(IrType::Void);
                        kit_mut.context_mut.set_return_type(Type::NotForce);
                    }
                    FuncType::Int => {
                        func_mut.set_return_type(IrType::Int);
                        kit_mut.context_mut.set_return_type(Type::Int);
                    }
                    FuncType::Float => {
                        func_mut.set_return_type(IrType::Float);
                        kit_mut.context_mut.set_return_type(Type::Float);
                    }
                }

                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                let params_vec = params.process(1, kit_mut).unwrap();

                kit_mut.context_mut.bb_now_set(bb); //和for语句顺序改过

                for (name, param) in params_vec {
                    kit_mut.context_mut.update_var_scope_now(&name, param); //新增
                    func_mut.set_parameter(name, param); //这里
                }

                blk.process((None, None), kit_mut).unwrap();
                kit_mut.context_mut.delete_layer();
                return Ok(func_ptr);
            }
        }
    }
}

impl Process for FuncFParams {
    type Ret = Vec<(String, ObjPtr<Inst>)>;
    type Message = i32;
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
    type Message = i32;
    fn process(&mut self, _: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            FuncFParam::Array((tp, id, vec)) => {
                //vec中存储的是从第二维开始的维度信息，第一维默认存在且为空
                match tp {
                    BType::Int => {
                        let param = kit_mut.pool_inst_mut.make_param(IrType::IntPtr);
                        let mut dimension_vec = vec![];
                        let mut dimension_vec_in = vec![];
                        for exp in vec {
                            dimension_vec.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        dimension_vec_in.push(-1);
                        for (_, dimension) in dimension_vec {
                            match dimension {
                                ExpValue::Int(i) => {
                                    dimension_vec_in.push(i);
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        kit_mut.context_mut.add_var(
                            id,
                            Type::Int,
                            true,
                            true,
                            Some(param),
                            None,
                            dimension_vec_in,
                        );
                        kit_mut.context_mut.update_var_scope_now(id, param);
                        Ok((id.clone(), param))
                    }
                    BType::Float => {
                        let param = kit_mut.pool_inst_mut.make_param(IrType::FloatPtr);
                        let mut dimension_vec = vec![];
                        let mut dimension_vec_in = vec![];
                        for exp in vec {
                            dimension_vec.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        dimension_vec_in.push(-1);
                        for (_, dimension) in dimension_vec {
                            match dimension {
                                ExpValue::Int(i) => {
                                    dimension_vec_in.push(i);
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        kit_mut.context_mut.add_var(
                            id,
                            Type::Float,
                            true,
                            true,
                            Some(param),
                            None,
                            dimension_vec_in,
                        );
                        //这里
                        kit_mut.context_mut.update_var_scope_now(id, param);
                        Ok((id.clone(), param))
                    }
                }
            }
            FuncFParam::NonArray((tp, id)) => match tp {
                BType::Int => {
                    let param = kit_mut.pool_inst_mut.make_param(IrType::Int);
                    kit_mut
                        .context_mut
                        .add_var(id, Type::Int, false, true, None, None, Vec::new());
                    //这里
                    kit_mut.context_mut.update_var_scope_now(id, param);
                    Ok((id.clone(), param))
                }
                BType::Float => {
                    let param = kit_mut.pool_inst_mut.make_param(IrType::Float);
                    kit_mut.context_mut.add_var(
                        id,
                        Type::Float,
                        false,
                        true,
                        None,
                        None,
                        Vec::new(),
                    );
                    kit_mut.context_mut.update_var_scope_now(id, param);
                    Ok((id.clone(), param))
                }
            },
        }
    }
}
impl Process for Block {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if kit_mut.context_mut.stop_genir {
            Ok(1)
        } else {
            kit_mut.context_mut.add_layer();
            for item in &mut self.block_vec {
                item.process(input, kit_mut).unwrap();
            }
            kit_mut.context_mut.delete_layer();
            Ok(1)
        }
    }
}

impl Process for BlockItem {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if kit_mut.context_mut.stop_genir {
            Ok(1)
        } else {
            match self {
                BlockItem::Decl(decl) => {
                    decl.process(1, kit_mut).unwrap();
                    return Ok(1);
                }
                BlockItem::Stmt(stmt) => {
                    stmt.process(input, kit_mut).unwrap();
                    return Ok(1);
                }
            }
        }
    }
}
impl Process for Stmt {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if kit_mut.context_mut.stop_genir {
            Ok(1)
        } else {
            match self {
                Stmt::Assign(assign) => assign.process(input, kit_mut),
                Stmt::ExpStmt(exp_stmt) => exp_stmt.process((Type::Int, input.0, input.1), kit_mut),
                Stmt::Block(blk) => blk.process(input, kit_mut),
                Stmt::If(if_stmt) => if_stmt.process(input, kit_mut),
                Stmt::While(while_stmt) => while_stmt.process(input, kit_mut),
                Stmt::Break(break_stmt) => {
                    break_stmt.process(input, kit_mut).unwrap();
                    kit_mut.context_mut.set_stop_genir(true);
                    Ok(1)
                }
                Stmt::Continue(continue_stmt) => {
                    continue_stmt.process(input, kit_mut).unwrap();
                    kit_mut.context_mut.set_stop_genir(true);
                    Ok(1)
                }
                Stmt::Return(ret_stmt) => {
                    ret_stmt.process(input, kit_mut).unwrap();
                    kit_mut.context_mut.set_stop_genir(true);
                    Ok(1)
                }
            }
        }
    }
}

impl Process for Assign {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, _: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let lval = &mut self.lval;
        let symbol = kit_mut.get_var_symbol(&lval.id).unwrap();
        let flag = self.exp.type_process(1, kit_mut).unwrap();
        let mut tp = Type::Int;
        if flag > 1 {
            tp = Type::Float;
        }
        let (mut inst_r, val) = self.exp.process(tp, kit_mut).unwrap();
        match symbol.tp {
            Type::ConstFloat | Type::Float => {
                if flag <= 1 {
                    //inst_r是int，要求是float
                    match val {
                        ExpValue::Int(i) => {
                            //需转型且值确定,直接转
                            inst_r = kit_mut.pool_inst_mut.make_float_const(i as f32);
                            kit_mut.context_mut.push_inst_bb(inst_r);
                        }
                        _ => {
                            inst_r = kit_mut.pool_inst_mut.make_int_to_float(inst_r);
                            kit_mut.context_mut.push_inst_bb(inst_r);
                        }
                    }
                }
            }
            Type::ConstInt | Type::Int => {
                if flag > 1 {
                    //inst_r是float，要求是int
                    match val {
                        ExpValue::Float(f) => {
                            //需转型且值确定,直接转
                            inst_r = kit_mut.pool_inst_mut.make_int_const(f as i32);
                            kit_mut.context_mut.push_inst_bb(inst_r);
                        }
                        _ => {
                            inst_r = kit_mut.pool_inst_mut.make_float_to_int(inst_r);
                            kit_mut.context_mut.push_inst_bb(inst_r);
                        }
                    }
                }
            }
            _ => {
                todo!()
            }
        }
        //函数参数不再单独处理
        if let Some(array_inst) = symbol.array_inst {
            //如果是数组
            if symbol.layer < 0 {
                match symbol.tp {
                    Type::ConstFloat | Type::Float => {
                        let ptr = kit_mut
                            .pool_inst_mut
                            .make_global_float_array_load(array_inst);
                        kit_mut.context_mut.push_inst_bb(ptr);
                        let inst_offset = offset_calculate(&lval.id, &mut lval.exp_vec, kit_mut);
                        let inst_ptr = kit_mut.pool_inst_mut.make_gep(ptr, inst_offset);
                        // kit_mut.context_mut.push_inst_bb(inst_offset); //这里需要吗  不需要
                        kit_mut.context_mut.push_inst_bb(inst_ptr);
                        let inst_store = kit_mut.pool_inst_mut.make_float_store(inst_ptr, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                    Type::ConstInt | Type::Int => {
                        let ptr = kit_mut.pool_inst_mut.make_global_int_array_load(array_inst);
                        kit_mut.context_mut.push_inst_bb(ptr);
                        let inst_offset = offset_calculate(&lval.id, &mut lval.exp_vec, kit_mut);
                        let inst_ptr = kit_mut.pool_inst_mut.make_gep(ptr, inst_offset);
                        // kit_mut.context_mut.push_inst_bb(inst_offset); //这里需要吗  不需要
                        kit_mut.context_mut.push_inst_bb(inst_ptr);
                        let inst_store = kit_mut.pool_inst_mut.make_int_store(inst_ptr, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                    _ => {
                        todo!()
                    }
                }

                // }
                kit_mut
                    .context_mut
                    .update_var_scope_now(&self.lval.id, inst_r);
                Ok(1)
            } else {
                let inst_offset = offset_calculate(&lval.id, &mut lval.exp_vec, kit_mut);
                let inst_ptr = kit_mut.pool_inst_mut.make_gep(array_inst, inst_offset);
                // kit_mut.context_mut.push_inst_bb(inst_offset); //这里需要吗  不需要
                kit_mut.context_mut.push_inst_bb(inst_ptr);
                match symbol.tp {
                    Type::ConstFloat | Type::Float => {
                        let inst_store = kit_mut.pool_inst_mut.make_float_store(inst_ptr, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                    Type::ConstInt | Type::Int => {
                        let inst_store = kit_mut.pool_inst_mut.make_int_store(inst_ptr, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                    _ => {
                        todo!()
                    }
                }

                // }
                kit_mut
                    .context_mut
                    .update_var_scope_now(&self.lval.id, inst_r);
                Ok(1)
            }
            //全局变量
        } else {
            //不是数组
            if let Some(global_inst) = symbol.global_inst {
                //全局变量
                match symbol.tp {
                    Type::ConstFloat | Type::Float => {
                        let inst_store =
                            kit_mut.pool_inst_mut.make_float_store(global_inst, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                    Type::ConstInt | Type::Int => {
                        let inst_store = kit_mut.pool_inst_mut.make_int_store(global_inst, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            kit_mut
                .context_mut
                .update_var_scope_now(&self.lval.id, inst_r);
            Ok(1)
        }
    }
}
impl Process for ExpStmt {
    type Ret = i32;
    type Message = (Type, Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if let Some(exp) = &mut self.exp {
            exp.process(input.0, kit_mut).unwrap();
        }
        Ok(1)
    }
}

impl Process for If {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let bb_if_name = kit_mut.context_mut.get_newbb_name();
        let inst_bb_if = kit_mut.pool_bb_mut.new_basic_block(bb_if_name.clone());

        if let Some(stmt_else) = &mut self.else_then {
            //如果有else语句
            let bb_else_name = kit_mut.context_mut.get_newbb_name();
            let inst_bb_else = kit_mut.pool_bb_mut.new_basic_block(bb_else_name.clone());

            let (_, _) = self
                .cond
                .process((Type::Int, Some(inst_bb_if), Some(inst_bb_else)), kit_mut)
                .unwrap();

            //生成一块新的bb
            let bb_successor_name = kit_mut.context_mut.get_newbb_name();
            let inst_bb_successor = kit_mut
                .pool_bb_mut
                .new_basic_block(bb_successor_name.clone());
            kit_mut.context_mut.bb_now_set(inst_bb_else); //设置现在所在的bb块，准备归约
            stmt_else.process(input, kit_mut).unwrap(); //向该分支块内生成指令
                                                        //加一条直接跳转语句

            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    if !kit_mut.context_mut.stop_genir {
                        kit_mut
                            .context_mut
                            .push_inst_bb(kit_mut.pool_inst_mut.make_jmp()); //bb_mut_now是else分支的叶子交汇点
                        bb_now.as_mut().add_next_bb(inst_bb_successor); //向if分支的叶子交汇点bb_now_mut插入下一个节点
                    }
                }
                _ => {
                    unreachable!()
                }
            }

            kit_mut.context_mut.set_stop_genir(false);
            kit_mut.context_mut.bb_now_set(inst_bb_if);
            self.then.process(input, kit_mut).unwrap();
            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    if !kit_mut.context_mut.stop_genir {
                        kit_mut
                            .context_mut
                            .push_inst_bb(kit_mut.pool_inst_mut.make_jmp()); //bb_now_mut是if语句块的叶子交汇点
                        bb_now.as_mut().add_next_bb(inst_bb_successor); //向if分支的叶子交汇点bb_now_mut插入下一个节点
                    }
                }
                _ => {
                    unreachable!()
                }
            }
            kit_mut.context_mut.set_stop_genir(false);
            if inst_bb_successor.get_up_bb().is_empty() {
                kit_mut.context_mut.set_stop_genir(true);
            }
            kit_mut.context_mut.bb_now_set(inst_bb_successor); //设置现在所在的bb
        } else {
            //没有else语句块
            //如果指定了该节点分支前的后继块
            //生成一块新的bb
            let bb_successor_name = kit_mut.context_mut.get_newbb_name();
            let inst_bb_successor = kit_mut
                .pool_bb_mut
                .new_basic_block(bb_successor_name.clone());

            let (_, _) = self
                .cond
                .process(
                    (Type::Int, Some(inst_bb_if), Some(inst_bb_successor)),
                    kit_mut,
                )
                .unwrap();

            kit_mut.context_mut.bb_now_set(inst_bb_if);
            self.then.process(input, kit_mut).unwrap();

            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    if !kit_mut.context_mut.stop_genir {
                        kit_mut
                            .context_mut
                            .push_inst_bb(kit_mut.pool_inst_mut.make_jmp()); //bb_now_mut是if语句块的叶子交汇点
                        bb_now.as_mut().add_next_bb(inst_bb_successor); //向if分支的叶子交汇点bb_now_mut插入下一个节点
                    }
                }
                _ => {
                    unreachable!()
                }
            }
            kit_mut.context_mut.set_stop_genir(false);
            if inst_bb_successor.get_up_bb().is_empty() {
                kit_mut.context_mut.set_stop_genir(true);
            } //没必要,可以继续翻译，但是该块不和任何块相连接
            kit_mut.context_mut.bb_now_set(inst_bb_successor); //设置现在所在的bb
        }
        Ok(1)
    }
}
impl Process for While {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let block_while_head_name = kit_mut.context_mut.get_newbb_name();
        let block_while_head = kit_mut.pool_bb_mut.new_basic_block(block_while_head_name); //生成新的块(false)
        let block_false_name = kit_mut.context_mut.get_newbb_name();
        let block_false = kit_mut.pool_bb_mut.new_basic_block(block_false_name); //生成新的块(false)
        let block_cond_name = kit_mut.context_mut.get_newbb_name();
        let block_cond = kit_mut.pool_bb_mut.new_basic_block(block_cond_name);
        let inst_jmp = kit_mut.pool_inst_mut.make_jmp();
        kit_mut.context_mut.push_inst_bb(inst_jmp);
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                bb_now.as_mut().add_next_bb(block_cond);
            }
            _ => {
                unreachable!()
            }
        }
        kit_mut.context_mut.bb_now_set(block_cond);

        let (_, _) = self
            .cond
            .process(
                (Type::Int, Some(block_while_head), Some(block_false)),
                kit_mut,
            )
            .unwrap();
        kit_mut.context_mut.bb_now_set(block_while_head); //设置当前basicblock
        self.body
            .process((Some(block_cond), Some(block_false)), kit_mut)
            .unwrap(); //在块内生成指令
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                // if !bb_now.get_up_bb().is_empty() {
                if !kit_mut.context_mut.stop_genir {
                    //有前继到达汇合点
                    bb_now.as_mut().add_next_bb(block_cond);
                    // println!(
                    //     "last_inst:{:?},stopflag{:?}",
                    //     bb_now.get_tail_inst().get_kind(),
                    //     kit_mut.context_mut.stop_genir
                    // );
                    let inst_jmp = kit_mut.pool_inst_mut.make_jmp();
                    kit_mut.context_mut.push_inst_bb(inst_jmp);
                }
                //有前继到达汇合点
                // bb_now.as_mut().add_next_bb(block_cond);
                // println!(
                //     "last_inst:{:?},stopflag{:?}",
                //     bb_now.get_tail_inst().get_kind(),
                //     kit_mut.context_mut.stop_genir
                // );
                // let inst_jmp = kit_mut.pool_inst_mut.make_jmp();
                // kit_mut.context_mut.push_inst_bb(inst_jmp);
            }
            _ => {
                unreachable!()
            }
        }
        kit_mut.context_mut.set_stop_genir(false);
        kit_mut.context_mut.bb_now_set(block_false);
        Ok(1)
    }
}

impl Process for Break {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let inst_jmp = kit_mut.pool_inst_mut.make_jmp();
        kit_mut.context_mut.push_inst_bb(inst_jmp);
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                let (_, false_opt) = input;
                if let Some(bb_false) = false_opt {
                    if bb_now.get_next_bb().len() <= 1 {
                        bb_now.as_mut().add_next_bb(bb_false);
                    }
                }
                kit_mut
                    .context_mut
                    .is_terminated_map
                    .insert(bb_now.get_name().to_string(), true);
            }
            _ => {
                unreachable!()
            }
        }
        Ok(1)
    }
}
impl Process for Continue {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let inst_jmp = kit_mut.pool_inst_mut.make_jmp();
        kit_mut.context_mut.push_inst_bb(inst_jmp);
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                let (head_opt, _) = input;
                if let Some(bb_false) = head_opt {
                    if bb_now.get_next_bb().len() <= 1 {
                        bb_now.as_mut().add_next_bb(bb_false);
                    }
                }
                kit_mut
                    .context_mut
                    .is_terminated_map
                    .insert(bb_now.get_name().to_string(), true);
            }
            _ => {
                unreachable!()
            }
        }
        Ok(1)
    }
}

impl Process for Return {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                kit_mut
                    .context_mut
                    .is_terminated_map
                    .insert(bb_now.get_name().to_string(), true);
            }
            _ => {
                unreachable!()
            }
        }
        if let Some(exp) = &mut self.exp {
            let flag = exp.type_process(1, kit_mut).unwrap();
            let mut tp = Type::Int;
            if flag > 1 {
                tp = Type::Float;
            }
            let mut flag_trans = false;
            let (mut inst, val) = exp.process(tp, kit_mut).unwrap(); //这里可能有问题
            match kit_mut.context_mut.ret_tp {
                Type::Int => {
                    if flag > 1 {
                        flag_trans = true;
                        inst = kit_mut.pool_inst_mut.make_float_to_int(inst);
                        kit_mut.context_mut.push_inst_bb(inst); //不管是否冗余先放进去
                    }
                }
                Type::Float => {
                    if flag <= 1 {
                        flag_trans = true;
                        inst = kit_mut.pool_inst_mut.make_int_to_float(inst);
                        kit_mut.context_mut.push_inst_bb(inst);
                    }
                }
                _ => {}
            }

            match val {
                ExpValue::Float(f) => {
                    let mut inst_float = kit_mut.pool_inst_mut.make_float_const(f);
                    if flag_trans {
                        inst_float = kit_mut.pool_inst_mut.make_int_const(f as i32);
                    }
                    let ret_inst = kit_mut.pool_inst_mut.make_return(inst_float);
                    kit_mut.context_mut.push_inst_bb(inst_float);
                    kit_mut.context_mut.push_inst_bb(ret_inst);
                    match kit_mut.context_mut.bb_now_mut {
                        InfuncChoice::InFunc(bb_now) => {
                            let func_now = &kit_mut.context_mut.func_now;
                            if let Some(vec) = kit_mut.context_mut.terminated_map.get_mut(func_now)
                            {
                                vec.push((bb_now, inst_float));
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                ExpValue::Int(i) => {
                    let mut inst_int = kit_mut.pool_inst_mut.make_int_const(i);
                    if flag_trans {
                        inst_int = kit_mut.pool_inst_mut.make_float_const(i as f32);
                    }
                    let ret_inst = kit_mut.pool_inst_mut.make_return(inst_int);
                    kit_mut.context_mut.push_inst_bb(inst_int);
                    kit_mut.context_mut.push_inst_bb(ret_inst);
                    match kit_mut.context_mut.bb_now_mut {
                        InfuncChoice::InFunc(bb_now) => {
                            let func_now = &kit_mut.context_mut.func_now;
                            if let Some(vec) = kit_mut.context_mut.terminated_map.get_mut(func_now)
                            {
                                vec.push((bb_now, inst_int));
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                ExpValue::None => {
                    let ret_inst = kit_mut.pool_inst_mut.make_return(inst);
                    kit_mut.context_mut.push_inst_bb(ret_inst);
                    match kit_mut.context_mut.bb_now_mut {
                        InfuncChoice::InFunc(bb_now) => {
                            let func_now = &kit_mut.context_mut.func_now;
                            if let Some(vec) = kit_mut.context_mut.terminated_map.get_mut(func_now)
                            {
                                vec.push((bb_now, inst));
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                _ => {
                    unreachable!()
                }
            }
            Ok(1)
        } else {
            let ret_inst = kit_mut.pool_inst_mut.make_return_void();
            kit_mut.context_mut.push_inst_bb(ret_inst);
            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    let func_now = &kit_mut.context_mut.func_now;
                    if let Some(vec) = kit_mut.context_mut.terminated_map.get_mut(func_now) {
                        vec.push((bb_now, kit_mut.pool_inst_mut.make_int_const(-1129)));
                    }
                }
                _ => {
                    unreachable!()
                }
            }

            Ok(1)
        }
    }
}
impl Process for Exp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.add_exp.process(input, kit_mut)
    }
}

impl Process for Cond {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type, Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>); //第一个为true,第二个为false
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if kit_mut.context_mut.stop_genir {
            Ok((kit_mut.pool_inst_mut.make_int_const(-1129), ExpValue::None))
        } else {
            self.l_or_exp
                .process((Type::NotForce, input.1, input.2), kit_mut)
        }
    }
}
impl Process for LVal {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let (mut var, mut symbol) = (
            kit_mut.pool_inst_mut.make_int_const(0),
            Symbol {
                tp: Type::Int,
                is_array: false,
                is_param: false,
                array_inst: None,
                global_inst: None,
                layer: -2,
                dimension: vec![],
            },
        );
        let sym_tmp = kit_mut.get_var_symbol(&self.id).unwrap();
        if self.exp_vec.is_empty() {
            //如果为空
            if sym_tmp.dimension.len() > self.exp_vec.len() {
                (var, symbol) = kit_mut.get_var(&self.id, None, true).unwrap();
            } else {
                // let inst_offset_temp = kit_mut.pool_inst_mut.make_int_const(0);
                (var, symbol) = kit_mut.get_var(&self.id, None, false).unwrap();
            }
        } else {
            let sym = kit_mut.get_var_symbol(&self.id).unwrap(); //获得符号表
            let dimension_vec = sym.dimension.clone(); //获得维度信息
            let mut index = 1;
            let mut inst_base_vec = vec![];
            for _ in dimension_vec {
                let mut after = 1;
                for i in index..sym.dimension.len() {
                    after = after * sym.dimension[i]; //计算该维度每加1对应多少元素
                }
                index = index + 1;
                inst_base_vec.push(after as i32);
                //vec存储维度base信息
            }
            let mut inst_add_vec = vec![];
            let mut imm_flag = true;
            index = 0;
            for exp in &mut self.exp_vec {
                let (inst_exp, val) = exp.process(Type::Int, kit_mut).unwrap();
                match val {
                    ExpValue::Int(i) => {
                        let inst_base_now =
                            kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                        let inst_oprand = kit_mut
                            .pool_inst_mut
                            .make_int_const(i * inst_base_vec[index]);
                        inst_add_vec.push((
                            inst_base_now,
                            inst_oprand,
                            ExpValue::Int(i * inst_base_vec[index]),
                        ))
                        //将inst和数push入vec中
                    }
                    ExpValue::Float(_) => {
                        unreachable!()
                    }
                    ExpValue::None => {
                        imm_flag = false; //设置flag,表示总偏移不再是一个可以确认的值
                        let inst_base_now =
                            kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                        let inst_oprand = kit_mut.pool_inst_mut.make_mul(inst_exp, inst_base_now); //构造这一维的偏移的inst
                        inst_add_vec.push((inst_base_now, inst_oprand, ExpValue::None));
                        //将inst和数push入vec中
                    }
                    _ => {
                        unreachable!()
                    }
                }
                index = index + 1;
            }
            let mut inst_offset = kit_mut.pool_inst_mut.make_int_const(-1129);
            let mut inst_base_now = kit_mut.pool_inst_mut.make_int_const(-1129);
            if imm_flag {
                //总偏移是一个可以计算出的值
                let mut offset_final = 0;
                for (_, _, add_val) in inst_add_vec {
                    match add_val {
                        ExpValue::Int(i) => {
                            offset_final = offset_final + i;
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                inst_offset = kit_mut.pool_inst_mut.make_int_const(offset_final);
                kit_mut.context_mut.push_inst_bb(inst_offset);
            } else {
                //总偏移不是是一个可以计算出的值
                (inst_base_now, inst_offset, _) = inst_add_vec[0];
                kit_mut.context_mut.push_inst_bb(inst_base_now);
                kit_mut.context_mut.push_inst_bb(inst_offset);
                for i in 1..inst_add_vec.len() {
                    kit_mut.context_mut.push_inst_bb(inst_add_vec[i].0); //每一维的基数被push进basicblock中
                    kit_mut.context_mut.push_inst_bb(inst_add_vec[i].1); //被加数push进basicblock中
                    inst_offset = kit_mut
                        .pool_inst_mut
                        .make_add(inst_offset, inst_add_vec[i].1); //构造新的add指令，左操作数改变
                    kit_mut.context_mut.push_inst_bb(inst_offset); //add指令push进basicblock中
                }
            }

            if sym_tmp.dimension.len() > self.exp_vec.len() {
                //获得指针
                (var, symbol) = kit_mut.get_var(&self.id, Some(inst_offset), true).unwrap();
            } else {
                (var, symbol) = kit_mut.get_var(&self.id, Some(inst_offset), false).unwrap();
            }
        }

        match input {
            Type::ConstFloat | Type::Float => match symbol.tp {
                Type::Int | Type::ConstInt => {
                    let inst_trans = kit_mut.pool_inst_mut.make_int_to_float(var);
                    match var.as_ref().get_kind() {
                        InstKind::Load => {
                            let mut val_ret = ExpValue::None;
                            match var.as_ref().get_ptr().as_ref().get_kind() {
                                InstKind::GlobalConstInt(i) => {
                                    val_ret = ExpValue::Float(i as f32);
                                }
                                _ => {}
                            }
                            kit_mut.context_mut.push_inst_bb(inst_trans);
                            return Ok((inst_trans, val_ret));
                        }
                        _ => {
                            let mut val_ret = ExpValue::None;
                            match var.as_ref().get_kind() {
                                InstKind::ConstInt(i)
                                | InstKind::GlobalInt(i)
                                | InstKind::GlobalConstInt(i) => {
                                    val_ret = ExpValue::Float(i as f32);
                                }
                                _ => {}
                            }
                            kit_mut.context_mut.push_inst_bb(inst_trans);
                            return Ok((inst_trans, val_ret));
                        }
                    }
                }
                _ => match var.as_ref().get_kind() {
                    InstKind::Load => {
                        let mut val_ret = ExpValue::None;
                        match var.as_ref().get_ptr().as_ref().get_kind() {
                            InstKind::GlobalConstFloat(f) => {
                                val_ret = ExpValue::Float(f);
                            }
                            _ => {}
                        }

                        return Ok((var, val_ret));
                    }
                    _ => {
                        let mut val_ret = ExpValue::None;
                        match var.as_ref().get_kind() {
                            InstKind::ConstFloat(f)
                            | InstKind::GlobalFloat(f)
                            | InstKind::GlobalConstFloat(f) => {
                                val_ret = ExpValue::Float(f);
                            }
                            _ => {}
                        }
                        return Ok((var, val_ret));
                    }
                },
            },
            Type::ConstInt | Type::Int => match symbol.tp {
                Type::Float | Type::ConstFloat => {
                    let inst_trans = kit_mut.pool_inst_mut.make_float_to_int(var);
                    match var.as_ref().get_kind() {
                        InstKind::Load => {
                            let mut val_ret = ExpValue::None;
                            kit_mut.context_mut.push_inst_bb(inst_trans);
                            match var.as_ref().get_ptr().as_ref().get_kind() {
                                InstKind::GlobalConstFloat(f) => {
                                    val_ret = ExpValue::Int(f as i32);
                                }
                                _ => {}
                            }
                            return Ok((inst_trans, val_ret));
                        }
                        _ => {
                            let mut val_ret = ExpValue::None;
                            match var.as_ref().get_kind() {
                                InstKind::ConstFloat(f)
                                | InstKind::GlobalFloat(f)
                                | InstKind::GlobalConstFloat(f) => {
                                    val_ret = ExpValue::Int(f as i32);
                                }
                                _ => {}
                            }

                            kit_mut.context_mut.push_inst_bb(inst_trans);
                            return Ok((inst_trans, val_ret));
                        }
                    }
                }
                _ => match var.as_ref().get_kind() {
                    InstKind::Load => {
                        let mut val_ret = ExpValue::None;

                        match var.as_ref().get_ptr().as_ref().get_kind() {
                            InstKind::GlobalConstInt(i) => {
                                val_ret = ExpValue::Int(i);
                            }
                            _ => {}
                        }

                        return Ok((var, val_ret));
                    }
                    _ => {
                        let mut val_ret = ExpValue::None;
                        match var.as_ref().get_kind() {
                            InstKind::ConstInt(i)
                            | InstKind::GlobalInt(i)
                            | InstKind::GlobalConstInt(i) => {
                                val_ret = ExpValue::Int(i);
                            }
                            _ => {}
                        }

                        return Ok((var, val_ret));
                    }
                },
            },
            _ => match symbol.tp {
                Type::Int | Type::ConstInt => match var.as_ref().get_kind() {
                    InstKind::Load => {
                        let mut val_ret = ExpValue::None;

                        match var.as_ref().get_ptr().as_ref().get_kind() {
                            InstKind::GlobalConstInt(i) => {
                                val_ret = ExpValue::Int(i);
                            }
                            _ => {}
                        }

                        return Ok((var, val_ret));
                    }
                    _ => {
                        let mut val_ret = ExpValue::None;
                        match var.as_ref().get_kind() {
                            InstKind::ConstInt(i)
                            | InstKind::GlobalInt(i)
                            | InstKind::GlobalConstInt(i) => {
                                val_ret = ExpValue::Int(i);
                            }
                            _ => {}
                        }

                        return Ok((var, val_ret));
                    }
                },
                Type::Float | Type::ConstFloat => match var.as_ref().get_kind() {
                    InstKind::Load => {
                        let mut val_ret = ExpValue::None;

                        match var.as_ref().get_ptr().as_ref().get_kind() {
                            InstKind::GlobalConstInt(i) => {
                                val_ret = ExpValue::Int(i);
                            }
                            _ => {}
                        }

                        return Ok((var, val_ret));
                    }
                    _ => {
                        let mut val_ret = ExpValue::None;
                        match var.as_ref().get_kind() {
                            InstKind::ConstInt(i)
                            | InstKind::GlobalInt(i)
                            | InstKind::GlobalConstInt(i) => {
                                val_ret = ExpValue::Int(i);
                            }
                            _ => {}
                        }

                        return Ok((var, val_ret));
                    }
                },
                _ => {
                    todo!()
                }
            },
        }
    }
}

impl Process for PrimaryExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type;
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
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Number::FloatConst(f) => {
                match input {
                    Type::ConstInt | Type::Int => {
                        // unreachable!("这里有问题,但是不可能到这");
                        let f = *f as i32;
                        if let Some(inst) = kit_mut.context_mut.get_const_int(f) {
                            return Ok((inst, ExpValue::Int(f)));
                        } else {
                            let inst = kit_mut.pool_inst_mut.make_int_const(f);
                            kit_mut.context_mut.add_const_int(f, inst);
                            return Ok((inst, ExpValue::Int(f)));
                        }
                    }
                    Type::ConstFloat | Type::Float | Type::NotForce => {
                        if let Some(inst) = kit_mut.context_mut.get_const_float(*f) {
                            return Ok((inst, ExpValue::Float(*f)));
                        } else {
                            let inst = kit_mut.pool_inst_mut.make_float_const(*f);
                            kit_mut.context_mut.add_const_float(*f, inst);
                            return Ok((inst, ExpValue::Float(*f)));
                        }
                    }
                }
            }
            Number::IntConst(i) => match input {
                Type::ConstFloat | Type::Float => {
                    let f = *i as f32;
                    if let Some(inst) = kit_mut.context_mut.get_const_float(f) {
                        return Ok((inst, ExpValue::Float(f)));
                    } else {
                        let inst = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.add_const_float(f, inst);
                        return Ok((inst, ExpValue::Float(f)));
                    }
                }
                Type::ConstInt | Type::Int | Type::NotForce => {
                    if let Some(inst) = kit_mut.context_mut.get_const_int(*i) {
                        return Ok((inst, ExpValue::Int(*i)));
                    } else {
                        let inst = kit_mut.pool_inst_mut.make_int_const(*i);
                        kit_mut.context_mut.add_const_int(*i, inst);
                        return Ok((inst, ExpValue::Int(*i)));
                    }
                }
            },
        }
    }
}

impl Process for OptionFuncRParams {
    type Ret = Vec<ObjPtr<Inst>>;
    type Message = Vec<Type>;
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
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            UnaryExp::PrimaryExp(primaryexp) => primaryexp.process(input, kit_mut),
            UnaryExp::OpUnary((unaryop, unaryexp)) => match unaryop {
                UnaryOp::Add => unaryexp.as_mut().process(input, kit_mut),
                UnaryOp::Minus => {
                    let (inst_u, val) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let mut inst = inst_u;

                    let mut val_ret = val;
                    match val {
                        ExpValue::Float(f) => {
                            val_ret = ExpValue::Float(-f);
                            inst = kit_mut.pool_inst_mut.make_float_const(-f);
                            kit_mut.context_mut.push_inst_bb(inst);
                            return Ok((inst, val_ret));
                        }
                        ExpValue::Int(i) => {
                            val_ret = ExpValue::Int(-i);
                            inst = kit_mut.pool_inst_mut.make_int_const(-i);
                            kit_mut.context_mut.push_inst_bb(inst);
                            return Ok((inst, val_ret));
                        }
                        ExpValue::Bool(i) => {
                            val_ret = ExpValue::Bool(-i);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    }

                    match inst_u.get_kind() {
                        InstKind::Unary(op) => match op {
                            UnOp::Neg => {
                                inst = inst_u.get_unary_operand();
                            }
                            _ => {
                                inst = kit_mut.pool_inst_mut.make_neg(inst_u);
                                kit_mut.context_mut.push_inst_bb(inst);
                            }
                        },
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_neg(inst_u);
                            kit_mut.context_mut.push_inst_bb(inst);
                        }
                    }
                    Ok((inst, val_ret))
                }
                UnaryOp::Exclamation => {
                    let (inst_u, val) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let mut inst = inst_u;
                    let mut val_ret = val;
                    match val {
                        //能推断出值
                        ExpValue::Float(f) => {
                            if f == 0.0 {
                                val_ret = ExpValue::Float(1.0);
                                inst = kit_mut.pool_inst_mut.make_float_const(1.0);
                                kit_mut.context_mut.push_inst_bb(inst);
                                return Ok((inst, val_ret));
                            } else {
                                val_ret = ExpValue::Float(0.0);
                                inst = kit_mut.pool_inst_mut.make_float_const(0.0);
                                kit_mut.context_mut.push_inst_bb(inst);
                                return Ok((inst, val_ret));
                            }
                        }
                        ExpValue::Int(i) => {
                            if i == 0 {
                                val_ret = ExpValue::Int(1);
                                inst = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst);
                                return Ok((inst, val_ret));
                            } else {
                                val_ret = ExpValue::Int(0);
                                inst = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst);
                                return Ok((inst, val_ret));
                            }
                        }
                        _ => {}
                    }
                    match inst_u.get_kind() {
                        InstKind::Unary(op) => match op {
                            UnOp::Not => {
                                inst = inst_u.get_unary_operand();
                                Ok((inst, ExpValue::None))
                            }
                            _ => {
                                inst = kit_mut.pool_inst_mut.make_not(inst_u);
                                kit_mut.context_mut.push_inst_bb(inst);
                                Ok((inst, ExpValue::None))
                            }
                        },
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_not(inst_u);
                            kit_mut.context_mut.push_inst_bb(inst);
                            Ok((inst, ExpValue::None))
                        }
                    }
                }
            },
            UnaryExp::FuncCall((funcname, funcparams)) => {
                let inst_func = kit_mut.context_mut.module_mut.get_function(&funcname);
                let fparams = inst_func.as_ref().get_parameter_list();
                let mut fparams_type_vec = vec![];
                for fp in fparams {
                    //获得各参数类型
                    match fp.as_ref().get_ir_type() {
                        IrType::Float => {
                            fparams_type_vec.push(Type::Float);
                        }
                        IrType::Int => {
                            fparams_type_vec.push(Type::Int);
                        }
                        IrType::FloatPtr | IrType::IntPtr => {
                            fparams_type_vec.push(Type::NotForce);
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                match inst_func.as_ref().get_return_type() {
                    //根据返回值类型生成call指令
                    IrType::Float => {
                        let args = funcparams.process(fparams_type_vec, kit_mut).unwrap(); //获得实参
                        let fname = funcname.clone();

                        // let mut fname = " ".to_string();

                        // if let Some((funcname_in, _)) =
                        //     kit_mut.context_mut.module_mut.get_function(&funcname)
                        // {
                        //     fname = funcname_in.clone();
                        // }
                        let mut inst = kit_mut.pool_inst_mut.make_float_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        match input {
                            Type::ConstInt | Type::Int => {
                                inst = kit_mut.pool_inst_mut.make_float_to_int(inst);
                                kit_mut.context_mut.push_inst_bb(inst);
                            }
                            _ => {}
                        }
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    IrType::Int => {
                        let args = funcparams.process(fparams_type_vec, kit_mut).unwrap();
                        let fname = funcname.clone();
                        // let mut fname = " ".to_string();
                        // if let Some((funcname_in, _)) = kit_mut
                        //     .context_mut
                        //     .module_mut
                        //     .function
                        //     .get_key_value(funcname)
                        // {
                        //     fname = funcname_in.clone();
                        // }
                        let mut inst = kit_mut.pool_inst_mut.make_int_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        match input {
                            Type::ConstFloat | Type::Float => {
                                inst = kit_mut.pool_inst_mut.make_int_to_float(inst);
                                kit_mut.context_mut.push_inst_bb(inst);
                            }
                            _ => {}
                        }
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    IrType::Void => {
                        let args = funcparams.process(fparams_type_vec, kit_mut).unwrap();
                        let fname = funcname.clone();
                        // let mut fname = " ".to_string();
                        // if let Some((funcname_in, _)) = kit_mut
                        //     .context_mut
                        //     .module_mut
                        //     .function
                        //     .get_key_value(funcname)
                        // {
                        //     fname = funcname_in.clone();
                        // }
                        let inst = kit_mut.pool_inst_mut.make_void_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }
}

impl Process for FuncRParams {
    type Ret = Vec<ObjPtr<Inst>>;
    type Message = Vec<Type>;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let mut vec = vec![];
        let mut index = 0;
        for i in &mut self.exp_vec {
            let flag = i.type_process(1, kit_mut).unwrap();
            let mut tp = Type::Int;
            if flag > 1 {
                tp = Type::Float;
            }
            let (mut inst, _) = i.process(tp, kit_mut).unwrap();
            match input[index] {
                Type::Float | Type::ConstFloat => {
                    if flag <= 1 {
                        inst = kit_mut.pool_inst_mut.make_int_to_float(inst);
                        kit_mut.context_mut.push_inst_bb(inst);
                    }
                }
                Type::ConstInt | Type::Int => {
                    if flag > 1 {
                        inst = kit_mut.pool_inst_mut.make_float_to_int(inst);
                        kit_mut.context_mut.push_inst_bb(inst);
                    }
                }
                _ => {}
            }
            vec.push(inst);
            index = index + 1;
        }
        Ok(vec)
    }
}

impl Process for MulExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            MulExp::UnaryExp(unaryexp) => unaryexp.process(input, kit_mut),
            MulExp::MulExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let mut inst = kit_mut.pool_inst_mut.make_int_const(-1129);

                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 * f2);
                            inst = kit_mut.pool_inst_mut.make_float_const(f1 * f2);
                        }
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_mul(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 * i2);
                            inst = kit_mut.pool_inst_mut.make_int_const(i1 * i2);
                        }
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_mul(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        inst = kit_mut.pool_inst_mut.make_mul(inst_left, inst_right);
                        val_ret = ExpValue::None;
                    }
                }
                kit_mut.context_mut.push_inst_bb(inst);
                Ok((inst, val_ret))
            }
            MulExp::DivExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let mut inst = kit_mut.pool_inst_mut.make_int_const(-1129);

                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 / f2);
                            inst = kit_mut.pool_inst_mut.make_float_const(f1 / f2);
                        }
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_div(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 / i2);
                            inst = kit_mut.pool_inst_mut.make_int_const(i1 / i2);
                        }
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_div(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        inst = kit_mut.pool_inst_mut.make_div(inst_left, inst_right);
                        val_ret = ExpValue::None;
                    }
                }
                kit_mut.context_mut.push_inst_bb(inst);
                Ok((inst, val_ret))
            }
            MulExp::ModExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let mut inst = kit_mut.pool_inst_mut.make_int_const(-1129);

                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 % f2);
                            inst = kit_mut.pool_inst_mut.make_float_const(f1 % f2);
                            unreachable!("浮点数求余")
                        }
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_rem(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 % i2);
                            inst = kit_mut.pool_inst_mut.make_int_const(i1 % i2);
                        }
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_rem(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        inst = kit_mut.pool_inst_mut.make_rem(inst_left, inst_right);
                        val_ret = ExpValue::None;
                    }
                }
                kit_mut.context_mut.push_inst_bb(inst);
                Ok((inst, val_ret))
            }
        }
    }
}

impl Process for AddExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            AddExp::MulExp(mulexp) => mulexp.as_mut().process(input, kit_mut),
            AddExp::OpExp((opexp, op, mulexp)) => match op {
                AddOp::Add => {
                    let (inst_left, lval) = opexp.process(input, kit_mut).unwrap();
                    let (inst_right, rval) = mulexp.process(input, kit_mut).unwrap();
                    let mut inst = kit_mut.pool_inst_mut.make_int_const(-1129);

                    let mut val_ret = lval;
                    match lval {
                        ExpValue::Float(f1) => match rval {
                            ExpValue::Float(f2) => {
                                val_ret = ExpValue::Float(f1 + f2);
                                inst = kit_mut.pool_inst_mut.make_float_const(f1 + f2);
                            }
                            _ => {
                                inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                                val_ret = ExpValue::None;
                            }
                        },
                        ExpValue::Int(i1) => match rval {
                            ExpValue::Int(i2) => {
                                val_ret = ExpValue::Int(i1 + i2);
                                inst = kit_mut.pool_inst_mut.make_int_const(i1 + i2);
                            }
                            _ => {
                                inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                                val_ret = ExpValue::None;
                            }
                        },
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    }
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok((inst, val_ret))
                }
                AddOp::Minus => {
                    let (inst_left, lval) = opexp.process(input, kit_mut).unwrap();
                    let (inst_right, rval) = mulexp.process(input, kit_mut).unwrap();
                    // let inst_right_neg = kit_mut.pool_inst_mut.make_neg(inst_right);
                    let mut inst = kit_mut.pool_inst_mut.make_int_const(-1129);
                    // kit_mut.context_mut.push_inst_bb(inst_right);

                    let mut val_ret = lval;
                    match lval {
                        ExpValue::Float(f1) => match rval {
                            ExpValue::Float(f2) => {
                                val_ret = ExpValue::Float(f1 - f2);
                                inst = kit_mut.pool_inst_mut.make_float_const(f1 - f2);
                            }
                            _ => {
                                inst = kit_mut.pool_inst_mut.make_sub(inst_left, inst_right);
                                val_ret = ExpValue::None;
                            }
                        },
                        ExpValue::Int(i1) => match rval {
                            ExpValue::Int(i2) => {
                                val_ret = ExpValue::Int(i1 - i2);
                                inst = kit_mut.pool_inst_mut.make_int_const(i1 - i2);
                            }
                            _ => {
                                inst = kit_mut.pool_inst_mut.make_sub(inst_left, inst_right);
                                val_ret = ExpValue::None;
                            }
                        },
                        _ => {
                            inst = kit_mut.pool_inst_mut.make_sub(inst_left, inst_right);
                            val_ret = ExpValue::None;
                        }
                    }
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok((inst, val_ret))
                }
            },
        }
    }
}

impl Process for RelExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            RelExp::AddExp(addexp) => addexp.process(input, kit_mut),
            RelExp::OpExp((relexp, op, addexp)) => {
                let tp1 = relexp.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
                let tp2 = addexp.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
                let mut tp = 0;
                if tp1 > tp2 {
                    tp = tp1;
                } else {
                    tp = tp2;
                }
                let mut val_ret = ExpValue::None;
                let mut tp_in = Type::Int;
                if tp > 1 {
                    tp_in = Type::Float;
                }
                let (mut inst_left, val_left) = relexp.process(tp_in, kit_mut).unwrap();
                let (mut inst_right, val_right) = addexp.process(tp_in, kit_mut).unwrap();
                let mut fflag = -1;
                let mut iflag = -1;
                let mut fvec = vec![];
                let mut ivec = vec![];
                match val_left {
                    ExpValue::Float(f) => {
                        inst_left = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                        fflag = fflag + 1;
                        fvec.push(f);
                    }
                    ExpValue::Int(i) => {
                        inst_left = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                        iflag = iflag + 1;
                        ivec.push(i);
                    }
                    _ => {}
                }
                match val_right {
                    ExpValue::Float(f) => {
                        inst_right = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                        fflag = fflag + 1;
                        fvec.push(f);
                    }
                    ExpValue::Int(i) => {
                        inst_right = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                        iflag = iflag + 1;
                        ivec.push(i);
                    }
                    _ => {}
                } //这里可以进一步优化,计算cond是否恒为真或假
                let mut inst_cond = kit_mut.pool_inst_mut.make_int_const(-1129);
                let mut result = -1;
                match op {
                    RelOp::Greater => {
                        if fflag == 1 {
                            if fvec[0] > fvec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else if iflag == 1 {
                            if ivec[0] > ivec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else {
                            inst_cond = kit_mut.pool_inst_mut.make_gt(inst_left, inst_right);
                            kit_mut.context_mut.push_inst_bb(inst_cond);
                        }
                    }
                    RelOp::GreaterOrEqual => {
                        if fflag == 1 {
                            if fvec[0] >= fvec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else if iflag == 1 {
                            if ivec[0] >= ivec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else {
                            inst_cond = kit_mut.pool_inst_mut.make_ge(inst_left, inst_right);
                            kit_mut.context_mut.push_inst_bb(inst_cond);
                        }
                    }
                    RelOp::Less => {
                        if fflag == 1 {
                            if fvec[0] < fvec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else if iflag == 1 {
                            if ivec[0] < ivec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else {
                            inst_cond = kit_mut.pool_inst_mut.make_lt(inst_left, inst_right);
                            kit_mut.context_mut.push_inst_bb(inst_cond);
                        }

                        // Ok((inst_cond, ExpValue::None))
                    }
                    RelOp::LessOrEqual => {
                        if fflag == 1 {
                            if fvec[0] <= fvec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else if iflag == 1 {
                            if ivec[0] <= ivec[1] {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(1);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 1;
                            } else {
                                inst_cond = kit_mut.pool_inst_mut.make_int_const(0);
                                kit_mut.context_mut.push_inst_bb(inst_cond);
                                result = 0;
                            }
                        } else {
                            inst_cond = kit_mut.pool_inst_mut.make_le(inst_left, inst_right);
                            kit_mut.context_mut.push_inst_bb(inst_cond);
                        }
                    }
                }
                match input {
                    Type::ConstFloat | Type::Float => {
                        if result == 1 {
                            inst_cond = kit_mut.pool_inst_mut.make_float_const(1.0);
                            kit_mut.context_mut.push_inst_bb(inst_cond);
                            val_ret = ExpValue::Float(1.0);
                        } else if result == 0 {
                            inst_cond = kit_mut.pool_inst_mut.make_float_const(0.0);
                            kit_mut.context_mut.push_inst_bb(inst_cond);
                            val_ret = ExpValue::Float(0.0);
                        } else {
                            inst_cond = kit_mut.pool_inst_mut.make_int_to_float(inst_cond);
                            kit_mut.context_mut.push_inst_bb(inst_cond);
                        }
                    }
                    _ => {
                        if result == 1 {
                            val_ret = ExpValue::Int(1);
                        } else if result == 0 {
                            val_ret = ExpValue::Int(0);
                        }
                    }
                }
                Ok((inst_cond, val_ret))
            }
        }
    }
}
impl Process for EqExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type; //第一个为true,第二个为false //if中默认给Type::Int
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // let tp = self.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
        match self {
            EqExp::RelExp(relexp) => {
                let (inst, _) = relexp.process(input, kit_mut).unwrap();
                Ok((inst, ExpValue::Bool(1))) //这里可以优化
            }
            EqExp::EqualExp((eqexp, relexp)) => {
                let tp1 = relexp.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
                let tp2 = eqexp.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
                let mut tp = 0;
                if tp1 > tp2 {
                    tp = tp1;
                } else {
                    tp = tp2;
                }
                let mut val_ret = ExpValue::None;
                let mut tp_in = Type::Int;
                if tp > 1 {
                    tp_in = Type::Float;
                }
                let mut fflag = -1;
                let mut iflag = -1;
                let mut fvec = vec![];
                let mut ivec = vec![];
                let (inst_left, val_left) = eqexp.process(tp_in, kit_mut).unwrap();
                let (inst_right, val_right) = relexp.process(tp_in, kit_mut).unwrap();
                match val_left {
                    ExpValue::Float(f) => {
                        fflag = fflag + 1;
                        fvec.push(f);
                    }
                    ExpValue::Int(i) => {
                        iflag = iflag + 1;
                        ivec.push(i);
                    }
                    _ => {}
                }
                match val_right {
                    ExpValue::Float(f) => {
                        fflag = fflag + 1;
                        fvec.push(f);
                    }
                    ExpValue::Int(i) => {
                        iflag = iflag + 1;
                        ivec.push(i);
                    }
                    _ => {}
                } //这里可以进一步优化,计算cond是否恒为真或假
                let mut inst_eq = kit_mut.pool_inst_mut.make_int_const(-1129); //不管是否冗余
                let mut result = -1;
                if fflag == 1 {
                    if fvec[0] == fvec[1] {
                        inst_eq = kit_mut.pool_inst_mut.make_int_const(1);
                        result = 1;
                    } else {
                        inst_eq = kit_mut.pool_inst_mut.make_int_const(0);
                        result = 0;
                    }
                } else if iflag == 1 {
                    if ivec[0] == ivec[1] {
                        inst_eq = kit_mut.pool_inst_mut.make_int_const(1);
                        result = 1;
                    } else {
                        inst_eq = kit_mut.pool_inst_mut.make_int_const(0);
                        result = 0;
                    }
                } else {
                    inst_eq = kit_mut.pool_inst_mut.make_eq(inst_left, inst_right);
                }
                match input {
                    Type::ConstFloat | Type::Float => {
                        if result == 1 {
                            inst_eq = kit_mut.pool_inst_mut.make_float_const(1.0);
                            val_ret = ExpValue::Float(1.0);
                        } else if result == 0 {
                            inst_eq = kit_mut.pool_inst_mut.make_float_const(0.0);
                            val_ret = ExpValue::Float(0.0);
                        } else {
                            kit_mut.context_mut.push_inst_bb(inst_eq); //算不出来,把之前的先放进去
                            inst_eq = kit_mut.pool_inst_mut.make_int_to_float(inst_eq);
                            val_ret = ExpValue::None;
                        }
                        kit_mut.context_mut.push_inst_bb(inst_eq);
                    }
                    _ => {
                        kit_mut.context_mut.push_inst_bb(inst_eq); //把之前的先放进去
                        if result == 1 {
                            val_ret = ExpValue::Int(1);
                        } else if result == 0 {
                            val_ret = ExpValue::Int(0);
                        }
                    }
                }
                Ok((inst_eq, val_ret))
            }
            EqExp::NotEqualExp((eqexp, relexp)) => {
                let tp1 = relexp.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
                let tp2 = eqexp.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
                let mut tp = 0;
                if tp1 > tp2 {
                    tp = tp1;
                } else {
                    tp = tp2;
                }
                let mut val_ret = ExpValue::None;
                let mut tp_in = Type::Int;
                if tp > 1 {
                    tp_in = Type::Float;
                }
                let mut fflag = -1;
                let mut iflag = -1;
                let mut fvec = vec![];
                let mut ivec = vec![];
                let (inst_left, val_left) = eqexp.process(tp_in, kit_mut).unwrap();
                let (inst_right, val_right) = relexp.process(tp_in, kit_mut).unwrap();
                match val_left {
                    ExpValue::Float(f) => {
                        fflag = fflag + 1;
                        fvec.push(f);
                    }
                    ExpValue::Int(i) => {
                        iflag = iflag + 1;
                        ivec.push(i);
                    }
                    _ => {}
                }
                match val_right {
                    ExpValue::Float(f) => {
                        fflag = fflag + 1;
                        fvec.push(f);
                    }
                    ExpValue::Int(i) => {
                        iflag = iflag + 1;
                        ivec.push(i);
                    }
                    _ => {}
                } //这里可以进一步优化,计算cond是否恒为真或假
                let mut inst_ne = kit_mut.pool_inst_mut.make_int_const(-1129);
                let mut result = -1;
                if fflag == 1 {
                    if fvec[0] != fvec[1] {
                        inst_ne = kit_mut.pool_inst_mut.make_int_const(1);
                        result = 1;
                    } else {
                        inst_ne = kit_mut.pool_inst_mut.make_int_const(0);
                        result = 0;
                    }
                } else if iflag == 1 {
                    if ivec[0] != ivec[1] {
                        inst_ne = kit_mut.pool_inst_mut.make_int_const(1);
                        result = 1;
                    } else {
                        inst_ne = kit_mut.pool_inst_mut.make_int_const(0);
                        result = 0;
                    }
                } else {
                    inst_ne = kit_mut.pool_inst_mut.make_ne(inst_left, inst_right);
                }
                match input {
                    Type::ConstFloat | Type::Float => {
                        if result == 1 {
                            inst_ne = kit_mut.pool_inst_mut.make_float_const(1.0);
                            val_ret = ExpValue::Float(1.0);
                        } else if result == 0 {
                            inst_ne = kit_mut.pool_inst_mut.make_float_const(0.0);
                            val_ret = ExpValue::Float(0.0);
                        } else {
                            kit_mut.context_mut.push_inst_bb(inst_ne); //算不出来,把之前的先放进去
                            inst_ne = kit_mut.pool_inst_mut.make_int_to_float(inst_ne);
                            val_ret = ExpValue::None;
                        }
                        kit_mut.context_mut.push_inst_bb(inst_ne);
                    }
                    _ => {
                        kit_mut.context_mut.push_inst_bb(inst_ne); //把之前的先放进去
                        if result == 1 {
                            val_ret = ExpValue::Int(1);
                        } else if result == 0 {
                            val_ret = ExpValue::Int(0);
                        }
                    }
                }
                Ok((inst_ne, val_ret))
            }
        }
    }
}

impl Process for LAndExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type, Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>); //第一个为true,第二个为false
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            LAndExp::EqExp(eqexp) => {
                let (mut inst_ret, val_ret) = eqexp.process(input.0, kit_mut).unwrap();
                match kit_mut.context_mut.bb_now_mut {
                    InfuncChoice::InFunc(bb_now) => {
                        if let Some(bb_false) = input.2 {
                            bb_now.as_mut().add_next_bb(bb_false);
                        }
                        if let Some(bb_true) = input.1 {
                            bb_now.as_mut().add_next_bb(bb_true);
                        }
                        //这里
                        match inst_ret.get_ir_type() {
                            IrType::Float => {
                                let inst_zero = kit_mut.pool_inst_mut.make_float_const(0.0);
                                inst_ret = kit_mut.pool_inst_mut.make_ne(inst_ret, inst_zero);
                                kit_mut.context_mut.push_inst_bb(inst_ret);
                                kit_mut.context_mut.push_inst_bb(inst_zero);
                            }
                            IrType::Int => {}
                            _ => {
                                unreachable!("应该是一个bool值(int或float)才对")
                            }
                        }
                        let inst_branch = kit_mut.pool_inst_mut.make_br(inst_ret);
                        kit_mut.context_mut.push_inst_bb(inst_branch);
                    }
                    _ => {
                        unreachable!()
                    }
                }
                Ok((inst_ret, val_ret))
            }
            LAndExp::AndExp((landexp, eqexp)) => {
                let name_bb_cond_right = kit_mut.context_mut.get_newbb_name();
                let inst_bb_cond_right = kit_mut.pool_bb_mut.new_basic_block(name_bb_cond_right); //创建一个新的bb
                let (_, _) = landexp
                    .process((input.0, Some(inst_bb_cond_right), input.2), kit_mut)
                    .unwrap();
                kit_mut.context_mut.bb_now_set(inst_bb_cond_right);
                let (mut inst_right, _) = eqexp.process(input.0, kit_mut).unwrap();
                kit_mut.context_mut.bb_now_set(inst_bb_cond_right);
                match kit_mut.context_mut.bb_now_mut {
                    InfuncChoice::InFunc(bb_now) => {
                        if let Some(bb_false) = input.2 {
                            bb_now.as_mut().add_next_bb(bb_false);
                        }
                        if let Some(bb_true) = input.1 {
                            bb_now.as_mut().add_next_bb(bb_true);
                        }
                    }
                    _ => {
                        unreachable!()
                    }
                }
                //这里
                match inst_right.get_ir_type() {
                    IrType::Float => {
                        let inst_zero = kit_mut.pool_inst_mut.make_float_const(0.0);
                        inst_right = kit_mut.pool_inst_mut.make_ne(inst_right, inst_zero);
                        kit_mut.context_mut.push_inst_bb(inst_zero);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                    }
                    IrType::Int => {}
                    _ => {
                        unreachable!("应该是一个bool值(int或float)才对")
                    }
                }

                let inst_branch = kit_mut.pool_inst_mut.make_br(inst_right);
                kit_mut.context_mut.push_inst_bb(inst_branch);

                Ok((inst_right, ExpValue::None)) //随便传一个回去
            }
        }
    }
}
impl Process for ConstExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = Type;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.add_exp.process(input, kit_mut)
    }
}

impl Process for LOrExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type, Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>); //第一个为true,第二个为false
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            LOrExp::LAndExp(landexp) => landexp.process(input, kit_mut),
            LOrExp::OrExp((lorexp, landexp)) => {
                let name_bb_cond_right = kit_mut.context_mut.get_newbb_name();
                let inst_bb_cond_right = kit_mut.pool_bb_mut.new_basic_block(name_bb_cond_right); //新建一个bb
                let (inst_left, _) = lorexp
                    .process((input.0, input.1, Some(inst_bb_cond_right)), kit_mut)
                    .unwrap();
                kit_mut.context_mut.bb_now_set(inst_bb_cond_right);
                let (_, _) = landexp.process(input, kit_mut).unwrap();
                Ok((inst_left, ExpValue::None)) //这里或许可以优化//这里随便填的
            }
        }
    }
}

pub fn offset_calculate(id: &str, exp_vec: &mut Vec<Exp>, kit_mut: &mut Kit) -> ObjPtr<Inst> {
    let (_, symbol) = (
        kit_mut.pool_inst_mut.make_int_const(0),
        Symbol {
            tp: Type::Int,
            is_array: false,
            is_param: false,
            array_inst: None,
            global_inst: None,
            layer: -2,
            dimension: vec![],
        },
    ); //初始化

    let sym = kit_mut.get_var_symbol(id).unwrap(); //获得符号表
    let dimension_vec = sym.dimension.clone(); //获得维度信息
    let mut index = 1;
    let mut inst_base_vec = vec![];
    for _ in dimension_vec {
        let mut after = 1;
        for i in index..sym.dimension.len() {
            after = after * sym.dimension[i]; //计算该维度每加1对应多少元素
        }
        index = index + 1;
        inst_base_vec.push(after as i32);
        //vec存储维度base信息
    }
    let mut inst_add_vec = vec![];
    let mut imm_flag = true;
    index = 0;
    for exp in exp_vec {
        let (inst_exp, val) = exp.process(symbol.tp, kit_mut).unwrap();
        match val {
            ExpValue::Int(i) => {
                let inst_base_now = kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                let inst_oprand = kit_mut
                    .pool_inst_mut
                    .make_int_const(i * inst_base_vec[index]);
                inst_add_vec.push((
                    inst_base_now,
                    inst_oprand,
                    ExpValue::Int(i * inst_base_vec[index]),
                ))
                //将inst和数push入vec中
            }
            ExpValue::Float(f) => {
                unreachable!()
            }
            ExpValue::None => {
                imm_flag = false; //设置flag,表示总偏移不再是一个可以确认的值
                let inst_base_now = kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                let inst_oprand = kit_mut.pool_inst_mut.make_mul(inst_exp, inst_base_now); //构造这一维的偏移的inst
                inst_add_vec.push((inst_base_now, inst_oprand, ExpValue::None));
                //将inst和数push入vec中
            }
            _ => {
                unreachable!()
            }
        }
        index = index + 1;
    }
    let mut inst_offset = kit_mut.pool_inst_mut.make_int_const(-1129);
    let mut inst_base_now = kit_mut.pool_inst_mut.make_int_const(-1129);
    if imm_flag {
        //总偏移是一个可以计算出的值
        let mut offset_final = 0;
        for (_, _, add_val) in inst_add_vec {
            match add_val {
                ExpValue::Int(i) => {
                    offset_final = offset_final + i;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        inst_offset = kit_mut.pool_inst_mut.make_int_const(offset_final);
        kit_mut.context_mut.push_inst_bb(inst_offset);
    } else {
        //总偏移不是是一个可以计算出的值
        (inst_base_now, inst_offset, _) = inst_add_vec[0];
        kit_mut.context_mut.push_inst_bb(inst_base_now);
        kit_mut.context_mut.push_inst_bb(inst_offset);
        for i in 1..inst_add_vec.len() {
            kit_mut.context_mut.push_inst_bb(inst_add_vec[i].0); //每一维的基数被push进basicblock中
            kit_mut.context_mut.push_inst_bb(inst_add_vec[i].1); //被加数push进basicblock中
            inst_offset = kit_mut
                .pool_inst_mut
                .make_add(inst_offset, inst_add_vec[i].1); //构造新的add指令，左操作数改变
            kit_mut.context_mut.push_inst_bb(inst_offset); //add指令push进basicblock中
        }
    }

    inst_offset
}
