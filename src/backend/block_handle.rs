use super::block::*;
impl BB {
    pub fn handle_spill(
        &mut self,
        func: ObjPtr<Func>,
        spill: &HashSet<i32>,
        pos: i32,
        pool: &mut BackendPool,
    ) {
        let mut start_pos = pos;
        let mut index = 0; 
        loop {
            if index >= self.insts.len() {
                break;
            }
            let inst = self.insts[index];
            let spills = inst.is_spill(spill);
            if spills.is_empty() {
                index += 1;
                continue;
            } else {
                let len = spills.len() as i32;
                let (mut store_num, mut slot) = (0, vec![]);
                for id in spills.iter() {
                    if let Some(offset) = func.spill_stack_map.get(&id) {
                        slot.push(*offset);
                    } else {
                        let stack_slot = StackSlot::new(start_pos + store_num * ADDR_SIZE, ADDR_SIZE);
                        slot.push(stack_slot);
                        store_num += 1;
                    };
                }
                let offset = start_pos + store_num * ADDR_SIZE;
                for i in 0..len {
                    let reg = Operand::Reg(Reg::new(5 + i, ScalarType::Int));
                    let stack_slot = slot[i as usize];
                    if func.spill_stack_map.contains_key(&spills[i as usize]) {
                        let mut ins = LIRInst::new(
                            InstrsType::LoadFromStack,
                            vec![reg, Operand::IImm(IImm::new(stack_slot.get_pos()))]
                        );
                        ins.set_double();
                        self.insts.insert(index, pool.put_inst(ins));
                        index += 1;
                    } else {
                        func.as_mut().spill_stack_map.insert(spills[i as usize], stack_slot);
                    }
                }
                for i in 0..len {
                    println!("{i}------------------");
                    // println!("{} inst: {:?}", self.label, self.insts[index-2]);
                    // println!("{} inst: {:?}", self.label, self.insts[index-1]);
                    println!("{} replace inst: {:?}", self.label, inst);
                    println!("------------------");
                    inst.as_mut().replace(spills[i as usize], 5 + i)
                }

                index += 1;

                for i in 0..len {
                    let reg = Operand::Reg(Reg::new(5 + i, ScalarType::Int));
                    let stack_slot = slot[i as usize];
                    match self.insts[index-1].get_dst() {
                        Operand::Reg(ireg) => {
                            if ireg.get_id() != 5 + i {
                                continue;
                            }
                        }
                        _ => {}
                    }
                    let mut ins = LIRInst::new(
                        InstrsType::StoreToStack,
                        vec![reg, Operand::IImm(IImm::new(stack_slot.get_pos()))]
                    );
                    ins.set_double();
                    self.insts.insert(index, pool.put_inst(ins));
                    index += 1;
                }

                start_pos = offset;
            }
        }
        println!("---------------------------");
        println!("{:?}", func.spill_stack_map);
        println!("---------------------------");
    }

    pub fn handle_overflow(&mut self, func: ObjPtr<Func>, pool: &mut BackendPool) {
        let mut pos = 0;
        loop {
            if pos >= self.insts.len() {
                break;
            }
            let inst_ref = self.insts[pos].as_ref();
            match inst_ref.get_type() {
                InstrsType::Load | InstrsType::Store => {
                    let temp = Operand::Reg(Reg::new(3, ScalarType::Int));
                    let offset = inst_ref.get_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        break;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    self.insts.insert(
                        pos,
                        pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Add),
                            vec![temp.clone(), temp.clone(), inst_ref.get_lhs().clone()],
                        )),
                    );
                    pos += 1;
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }
                InstrsType::LoadFromStack | InstrsType::StoreToStack => {
                    let temp = Operand::Reg(Reg::new(3, ScalarType::Int));
                    let offset = inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        break;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    self.insts.insert(
                        pos,
                        pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Add),
                            vec![
                                temp.clone(),
                                temp.clone(),
                                Operand::Reg(Reg::new(2, ScalarType::Int)),
                            ],
                        )),
                    );
                    pos += 1;
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }
                InstrsType::LoadParamFromStack | InstrsType::StoreParamToStack => {
                    let temp = Operand::Reg(Reg::new(3, ScalarType::Int));
                    let offset = func.as_ref().reg_alloc_info.stack_size as i32
                        - inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        break;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    self.insts.insert(
                        pos,
                        pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Add),
                            vec![
                                temp.clone(),
                                temp.clone(),
                                Operand::Reg(Reg::new(2, ScalarType::Int)),
                            ],
                        )),
                    );
                    pos += 1;
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }
                InstrsType::Branch(..) | InstrsType::Jump => {
                    // deal with false branch
                    let mut distance = 0;
                    let is_j = match inst_ref.get_type() {
                        InstrsType::Branch(..) => false,
                        InstrsType::Jump => true,
                        _ => unreachable!(),
                    };
                    let target = match inst_ref.get_label() {
                        Operand::Addr(label) => label,
                        _ => unreachable!("branch must have a label"),
                    };
                    let mut i = 0;
                    let (mut flag, mut first_j) = (false, true);
                    loop {
                        if i >= func.as_ref().blocks.len() {
                            break;
                        }
                        let block_ref = func.as_ref().blocks[i];
                        if &self.label == &block_ref.as_ref().label {
                            flag = true;
                        }
                        if &block_ref.as_ref().label == target {
                            break;
                        }
                        if flag {
                            distance += block_ref.as_ref().insts.len() * 4;
                        }
                        i += 1;
                        if (!is_j && !operand::is_imm_12bs(distance as i32))
                            || (is_j && !operand::is_imm_20bs(distance as i32))
                        {
                            let name = format!("overflow_{}", get_tmp_bb());
                            let tmp = pool.put_block(BB::new(&name));
                            func.as_mut().blocks.insert(i, tmp);
                            if first_j {
                                self.insts[pos].as_mut().replace_label(name);
                                if is_j {
                                    distance -= operand::IMM_20_Bs as usize;
                                } else {
                                    distance -= operand::IMM_12_Bs as usize;
                                }
                            } else {
                                self.insts.insert(
                                    pos,
                                    pool.put_inst(LIRInst::new(
                                        InstrsType::Jump,
                                        vec![Operand::Addr(name)],
                                    )),
                                );
                                distance -= operand::IMM_20_Bs as usize;
                            }
                            pos += 1;
                            first_j = false;
                        }
                    }
                }
                InstrsType::Call => {
                    // call 指令不会发生偏移量的溢出
                }
                _ => {}
            }
            pos += 1;
        }
    }

    pub fn resolve_overflow_sl(
        &mut self,
        temp: Operand,
        pos: &mut usize,
        offset: i32,
        pool: &mut BackendPool,
    ) {
        let op1 = Operand::IImm(IImm::new(offset >> 12));
        let op2 = Operand::IImm(IImm::new(offset & 0xfff));
        self.insts.insert(
            *pos,
            pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Lui),
                vec![temp.clone(), op1],
            )),
        );
        *pos += 1;
        self.insts.insert(
            *pos,
            pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Add),
                vec![temp.clone(), temp.clone(), op2],
            )),
        );
        *pos += 1;
    }

    pub fn resolve_operand(
        &mut self,
        func: ObjPtr<Func>,
        src: ObjPtr<Inst>,
        is_left: bool,
        map: &mut Mapping,
        pool: &mut BackendPool,
    ) -> Operand {
        if is_left {
            match src.as_ref().get_kind() {
                InstKind::ConstInt(iimm) => return self.load_iimm_to_ireg(iimm, pool),
                _ => {}
            }
        }

        match src.as_ref().get_kind() {
            InstKind::ConstInt(iimm) => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                self.resolve_iimm(iimm, pool)
            }
            InstKind::ConstFloat(fimm) => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                self.resolve_fimm(fimm, pool, func)
            }
            InstKind::Parameter => self.resolve_param(src, func, map, pool),
            InstKind::GlobalConstInt(_)
            | InstKind::GlobalInt(..)
            | InstKind::GlobalConstFloat(_)
            | InstKind::GlobalFloat(..) => self.resolve_global(src, map, pool),
            _ => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                let op: Operand = match src.as_ref().get_ir_type() {
                    IrType::Int | IrType::IntPtr => Operand::Reg(Reg::init(ScalarType::Int)),
                    IrType::Float | IrType::FloatPtr => Operand::Reg(Reg::init(ScalarType::Float)),
                    _ => unreachable!("cannot reach, resolve_operand func, false, pool"),
                };
                map.val_map.insert(src, op.clone());
                op
            }
        }
    }

    pub fn resolve_iimm(&mut self, imm: i32, pool: &mut BackendPool) -> Operand {
        let res = IImm::new(imm);
        if operand::is_imm_12bs(imm) {
            Operand::IImm(res)
        } else {
            self.load_iimm_to_ireg(imm, pool)
        }
    }

    pub fn resolve_fimm(&mut self, imm: f32, pool: &mut BackendPool, func: ObjPtr<Func>) -> Operand {
        let var_name = format!(
            "{label}_float{index}",
            label = func.as_ref().label,
            index = func.as_ref().floats.len()
        );
        func.as_mut().floats.push((var_name.clone(), imm));
        let reg = Operand::Reg(Reg::init(ScalarType::Float));
        let tmp = Operand::Reg(Reg::init(ScalarType::Int));
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::OpReg(SingleOp::LoadAddr),
            vec![tmp.clone(), Operand::Addr(var_name)],
        )));
        let mut inst = LIRInst::new(
            InstrsType::Load,
            vec![reg.clone(), tmp, Operand::IImm(IImm::new(0))],
        );
        inst.set_float();
        self.insts.push(pool.put_inst(inst));
        reg
    }

    pub fn load_iimm_to_ireg(&mut self, imm: i32, pool: &mut BackendPool) -> Operand {
        let reg = Operand::Reg(Reg::init(ScalarType::Int));
        let iimm = Operand::IImm(IImm::new(imm));
        if operand::is_imm_12bs(imm) {
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Li),
                vec![reg.clone(), iimm],
            )));
        } else {
            let op1 = Operand::IImm(IImm::new(imm >> 12));
            let op2 = Operand::IImm(IImm::new(imm & 0xfff));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Lui),
                vec![reg.clone(), op1],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Add),
                vec![reg.clone(), reg.clone(), op2],
            )));
        }
        reg
    }

    fn resolve_param(
        &mut self,
        src: ObjPtr<Inst>,
        func: ObjPtr<Func>,
        map: &mut Mapping,
        pool: &mut BackendPool,
    ) -> Operand {
        if !map.val_map.contains_key(&src) {
            let params = &func.as_ref().params;
            let reg = match src.as_ref().get_param_type() {
                IrType::Int => Operand::Reg(Reg::init(ScalarType::Int)),
                IrType::Float => Operand::Reg(Reg::init(ScalarType::Float)),
                _ => unreachable!("cannot reach, param either int or float"),
            };
            map.val_map.insert(src, reg.clone());
            let (mut inum, mut fnum) = (0, 0);
            for p in params {
                match p.as_ref().get_param_type() {
                    IrType::Int => {
                        if src == *p {
                            if inum < ARG_REG_COUNT {
                                let inst = LIRInst::new(
                                    InstrsType::OpReg(SingleOp::IMv),
                                    vec![
                                        reg.clone(),
                                        Operand::Reg(Reg::new(inum + 10, ScalarType::Int)),
                                    ],
                                );
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));
                            } else {
                                let inst = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![
                                        reg.clone(),
                                        Operand::IImm(IImm::new(
                                            inum - ARG_REG_COUNT + max(fnum - ARG_REG_COUNT, 0) * 4,
                                        )),
                                    ],
                                );
                                self.insts.push(pool.put_inst(inst));
                            }
                        }
                        inum += 1;
                    }
                    IrType::Float => {
                        if src == *p {
                            if fnum < ARG_REG_COUNT {
                                let inst = LIRInst::new(
                                    InstrsType::OpReg(SingleOp::FMv),
                                    vec![
                                        reg.clone(),
                                        Operand::Reg(Reg::new(fnum + 10, ScalarType::Float)),
                                    ],
                                );
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));
                            } else {
                                let inst = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![
                                        reg.clone(),
                                        Operand::IImm(IImm::new(
                                            fnum - ARG_REG_COUNT + max(inum - ARG_REG_COUNT, 0) * 4,
                                        )),
                                    ],
                                );
                                self.insts.push(pool.put_inst(inst));
                            }
                        }
                        fnum += 1;
                    }
                    _ => unreachable!("cannot reach, param either int or float"),
                }
            }
            reg
        } else {
            map.val_map.get(&src).unwrap().clone()
        }
    }

    pub fn resolve_global(
        &mut self,
        src: ObjPtr<Inst>,
        map: &mut Mapping,
        pool: &mut BackendPool,
    ) -> Operand {
        if !self.global_map.contains_key(&src) {
            let reg = match src.as_ref().get_ir_type() {
                IrType::Int => Operand::Reg(Reg::new(27, ScalarType::Int)),
                IrType::Float => Operand::Reg(Reg::init(ScalarType::Float)),
                _ => unreachable!("cannot reach, global var is either int or float"),
            };
            self.global_map.insert(src, reg.clone());
            // let global_num = get_current_global_seq();
            // self.label = String::from(format!(".Lpcrel_hi{global_num}"));
            // inc_global_seq();
            assert!(map.val_map.contains_key(&src));
            let global_name = match map.val_map.get(&src) {
                Some(Operand::Addr(addr)) => addr,
                _ => unreachable!("cannot reach, global var must be addr"),
            };
            let inst = LIRInst::new(
                InstrsType::OpReg(SingleOp::LoadAddr),
                vec![reg.clone(), Operand::Addr(global_name.clone())],
            );
            self.insts.push(pool.put_inst(inst));
            reg
        } else {
            println!("find!");
            return self.global_map.get(&src).unwrap().clone();
        }
    }

    pub fn resolve_opt_mul(&mut self, dst: Operand, src: Operand, imm: i32, pool: &mut BackendPool) {
        let abs = imm.abs();
        let is_neg = imm < 0;
        match abs {
            0 => {
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::OpReg(SingleOp::IMv),
                    vec![dst, Operand::IImm(IImm::new(0))],
                )));
            }
            1 => {
                if !is_neg {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::IMv),
                        vec![dst, src],
                    )));
                } else {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::INeg),
                        vec![dst, src],
                    )));
                }
            }
            _ => {
                if is_opt_num(abs) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![dst.clone(), src, Operand::IImm(IImm::new(log2(abs)))],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else if is_opt_num(abs - 1) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![
                            dst.clone(),
                            src.clone(),
                            Operand::IImm(IImm::new(log2(abs - 1))),
                        ],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![dst.clone(), dst.clone(), src],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else if is_opt_num(abs + 1) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![
                            dst.clone(),
                            src.clone(),
                            Operand::IImm(IImm::new(log2(abs + 1))),
                        ],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sub),
                        vec![dst.clone(), dst.clone(), src],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else {
                    let (mut power, mut opt_abs, mut do_add, mut can_opt) = (0, 0, false, false);
                    while (1 << power) <= abs {
                        if is_opt_num(abs + (1 << power)) {
                            do_add = true;
                            opt_abs = abs + (1 << power);
                            can_opt = true;
                            break;
                        }
                        if is_opt_num(abs - (1 << power)) {
                            opt_abs = abs - (1 << power);
                            can_opt = true;
                            break;
                        }
                        power += 1;
                    }
                    let temp = Operand::Reg(Reg::init(ScalarType::Int));
                    if !can_opt {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Li),
                            vec![temp.clone(), Operand::IImm(IImm::new(imm))],
                        )));
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Mul),
                            vec![dst, src, temp],
                        )));
                        return;
                    }
                    let bits = log2(opt_abs);
                    let combine_inst_kind = match do_add {
                        true => InstrsType::Binary(BinaryOp::Add),
                        false => InstrsType::Binary(BinaryOp::Sub),
                    };
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![temp.clone(), src.clone(), Operand::IImm(IImm::new(power))],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![dst.clone(), src.clone(), Operand::IImm(IImm::new(bits))],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        combine_inst_kind,
                        vec![dst.clone(), dst.clone(), temp],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                }
            }
        }
    }

    pub fn resolve_opt_div(&mut self, dst: Operand, src: Operand, imm: i32, pool: &mut BackendPool) {
        let abs = imm.abs();
        let is_neg = imm < 0;
        match abs {
            0 => {
                unreachable!("div by zero");
            }
            1 => {
                if is_neg {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::INeg),
                        vec![dst, src],
                    )))
                } else {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::IMv),
                        vec![dst, src],
                    )))
                }
            }
            _ => {
                if is_opt_num(abs) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sar),
                        vec![dst, src, Operand::IImm(IImm::new(log2(abs)))],
                    )))
                } else {
                    let (two31, uabs, mut p, mut delta) =
                        (1 << 31 as u32, abs as u32, 31, 0 as u32);
                    let t = two31 + (uabs >> 31);
                    let anc = t - 1 - t % uabs;
                    let (mut q1, mut q2) = (two31 / anc, two31 / uabs);
                    let (mut r1, mut r2) = (two31 - q1 * anc, two31 - q2 * uabs);

                    loop {
                        p += 1;
                        q1 *= 2;
                        r1 *= 2;

                        if r1 >= anc {
                            q1 += 1;
                            r1 -= anc;
                        }
                        q2 *= 2;
                        r2 *= 2;
                        if r2 >= uabs {
                            q2 += 1;
                            r2 -= uabs;
                        }
                        delta = uabs - r2;
                        if q1 < delta || (q1 == delta && r1 == 0) {
                            break;
                        }
                    }

                    let mut magic = (q2 + 1) as i32;
                    if is_neg {
                        magic = -magic;
                    }
                    let shift = p - 32;
                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Li),
                        vec![tmp.clone(), Operand::IImm(IImm::new(magic))],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Mulhs),
                        vec![tmp.clone(), tmp.clone(), src.clone()],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![tmp.clone(), src.clone(), tmp.clone()],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shr),
                        vec![tmp.clone(), tmp.clone(), Operand::IImm(IImm::new(shift))],
                    )));
                }
            }
        }
    }

    pub fn resolve_opt_rem(
        &mut self,
        func: ObjPtr<Func>,
        map: &mut Mapping,
        dst: Operand,
        lhs: ObjPtr<Inst>,
        imm: i32,
        pool: &mut BackendPool,
    ) {
        let lhs_reg = self.resolve_operand(func, lhs, true, map, pool);
        let abs = imm.abs();
        let is_neg = imm < 0;
        if is_opt_num(abs) {
            let k = log2(abs);
            // r = ((n + t) & (2^k - 1)) - t
            // t = (n >> k - 1) >> 32 - k
            let tmp = Operand::Reg(Reg::init(ScalarType::Int));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Sar),
                vec![
                    tmp.clone(),
                    lhs_reg.clone(),
                    Operand::IImm(IImm::new(k - 1)),
                ],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Shr),
                vec![tmp.clone(), tmp.clone(), Operand::IImm(IImm::new(32 - k))],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Add),
                vec![dst.clone(), dst.clone(), tmp.clone()],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::And),
                vec![dst.clone(), dst.clone(), Operand::IImm(IImm::new(abs - 1))],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Sub),
                vec![dst.clone(), dst.clone(), tmp.clone()],
            )));
        } else {
            let rhs_reg = self.load_iimm_to_ireg(imm, pool);
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Rem),
                vec![dst, lhs_reg, rhs_reg],
            )));
        }
    }
}

pub fn is_opt_mul(imm: i32) -> bool {
    //FIXME:暂时不使用优化
    false
}

pub fn is_opt_num(imm: i32) -> bool {
    //FIXME:暂时不使用优化
    (imm & (imm - 1)) == 0
}

pub fn log2(imm: i32) -> i32 {
    assert!(is_opt_num(imm));
    let mut res = 0;
    let mut tmp = imm;
    while tmp != 1 {
        tmp >>= 1;
        res += 1;
    }
    res
}

pub fn get_tmp_bb() -> i32 {
    unsafe {
        TMP_BB += 1;
        TMP_BB
    }
}

// fn get_current_global_seq() -> i32 {
//     unsafe { GLOBAL_SEQ }
// }

// fn inc_global_seq() {
//     unsafe {
//         GLOBAL_SEQ += 1;
//     }
// }

