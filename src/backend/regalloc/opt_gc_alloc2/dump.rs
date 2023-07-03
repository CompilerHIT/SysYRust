use crate::{log_file, log_file_uln};

use super::*;
impl Allocator {
    pub fn dump_action(&self, actions: &str, reg: &Reg) {
        match actions {
            "spill" => {
                log_file!("alloc_action.txt", "spill:{}", reg);
            }
            "color" => {
                log_file!(
                    "alloc_action.txt",
                    "color:{}({})",
                    reg,
                    self.get_color(&reg).unwrap()
                );
            }
            "decolor" => {
                log_file!(
                    "alloc_action.txt",
                    "decolor:{}({})",
                    reg,
                    self.get_color(&reg).unwrap(),
                );
            }
            "despill" => {
                log_file!("alloc_action.txt", "despill:{}", reg);
            }
            "simplify" => {
                log_file!("alloc_action.txt", "simplify:{}", reg);
            }
            "tocolor" => {
                log_file!("alloc_action.txt", "tocolor:{}", reg);
            }
            "tosimplify" => {
                log_file!("alloc_action.txt", "tosimplify:{}", reg);
            }
            "tospill" => {
                log_file!("alloc_action.txt", "tospill:{}", reg);
            }
            _ => unreachable!(),
        };
    }

    pub fn dump_color_action(&self, reg: &Reg, color: i32) {
        log_file!("alloc_action.txt", "color:{}({})", reg, color);
    }

    pub fn dump_swap_color(&self, reg1: &Reg, reg2: &Reg) {
        log_file!(
            "alloc_action.txt",
            "swap:{}({}),{}({})",
            reg1,
            self.get_color(reg1).unwrap(),
            reg2,
            self.get_color(reg2).unwrap()
        );
    }

    pub fn dump_last_colors(&self) {
        let p = "opt2.txt";
        log_file!(
            p,
            "last_colors,({}):\n{:?}",
            self.get_last_colors_lst().len(),
            self.info.as_ref().unwrap().last_colors_lst
        );
    }

    pub fn dump_all_neighbors(&self) {
        let intereref_path = "opt2.txt";
        log_file!(intereref_path, "all neighbors:");
        self.info
            .as_ref()
            .unwrap()
            .all_neighbors
            .iter()
            .for_each(|(reg, neighbors)| {
                log_file_uln!(intereref_path, "node {reg}\n num:{}{{", neighbors.len());
                neighbors
                    .iter()
                    .for_each(|neighbor| log_file_uln!(intereref_path, "({},{})", reg, neighbor));
                log_file!(intereref_path, "}}\n");
            });
    }

    pub fn dump_live_neighbors(&self, func: String) {
        let intereref_path = "opt2_live_graph.txt";
        log_file!(intereref_path, "func:{}:\nlive neighbors:", func);
        self.info
            .as_ref()
            .unwrap()
            .all_live_neighbors
            .iter()
            .for_each(|(reg, neighbors)| {
                log_file_uln!(intereref_path, "node {reg}\tnum:{}\n{{", neighbors.len());
                neighbors
                    .iter()
                    .for_each(|neighbor| log_file_uln!(intereref_path, "({},{})", reg, neighbor));
                log_file!(intereref_path, "}}\n");
            });
    }
    // dump to colors
    pub fn dump_tocolor(&self) {
        let p = "opt2.txt";
        log_file!(p, "to colors ({}):", self.get_tocolor().len());
        for item in self.info.as_ref().unwrap().to_color.iter() {
            log_file!(p, "{},", item.reg);
        }
    }

    pub fn dump_tosimplify(&self) {
        //
        let p = "opt2.txt";
        log_file!(p, "to colors:");
        for item in self.info.as_ref().unwrap().to_simplify.iter() {
            log_file!(p, "{},", item.reg);
        }
    }
    pub fn dump_tospill(&self) {
        let p = "opt2.txt";
        log_file!(p, "to colors:");
        for item in self.info.as_ref().unwrap().to_spill.iter() {
            log_file!(p, "{},", item.reg);
        }
    }

    // 查看colors
    pub fn dump_colors(&self) {
        let p = "opt2.txt";
        log_file!(p, "colors:\n{:?}", self.get_colors());
    }
    pub fn dump_spillings(&self) {
        let p = "opt2.txt";
        log_file!(p, "spillings:\n{:?}", self.get_colors());
    }

    pub fn dump_all(&self) {
        self.dump_last_colors();
        self.dump_colors();
        self.dump_spillings();
    }
}
