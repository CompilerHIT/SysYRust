// a impl of graph color register alloc algo

use crate::backend::operand::Reg;
use std::collections::{HashMap, HashSet};

use super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};

pub fn alloc(func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {
    let intereference_graph = regalloc::build_interference(func);
    let mut availables = regalloc::build_availables_with_interef_graph(&intereference_graph);
    let spill_costs = regalloc::estimate_spill_cost(func);
    let mut to_colors: Vec<Reg> = intereference_graph.iter().map(|(r, _)| *r).collect();
    let mut colors: HashMap<i32, i32> = HashMap::new();
    let mut spillings: HashSet<i32> = HashSet::new();
    //按照顺序进行着色
    to_colors.retain(|r| !r.is_physic());
    to_colors.sort_by_cached_key(|reg| {
        let mut val = *spill_costs.get(reg).unwrap();
        let nln = intereference_graph
            .get(reg)
            .unwrap()
            .iter()
            .filter(|reg| !reg.is_physic())
            .count();
        let nac = availables
            .get(reg)
            .unwrap()
            .num_available_regs(reg.get_type());
        let div = (nln as f32 - nac as f32);
        val = (val * 1000.0) / (div * 1000.0 + 1.0);
        (val * 1000000.0) as usize
    });
    //着色失败则spill
    // let mut to_color=
    for to_color in to_colors {
        let this_available = *availables.get(&to_color).unwrap();
        if !this_available.is_available(to_color.get_type()) {
            spillings.insert(to_color.get_id());
            continue;
        }
        let available = this_available
            .get_available_reg(to_color.get_type())
            .unwrap();
        colors.insert(to_color.get_id(), available);
        for nb in intereference_graph.get(&to_color).unwrap() {
            availables.get_mut(&nb).unwrap().use_reg(available);
        }
    }
    FuncAllocStat {
        spillings: spillings,
        dstr: colors,
    }
}
