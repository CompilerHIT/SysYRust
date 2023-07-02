use crate::log_file;

use super::*;
impl Allocator {
    pub fn dump_last_colors(&self) {
        let p = "opt2.txt";
        log_file!(
            p,
            "last_colors:\n{:?}",
            self.info.as_ref().unwrap().last_colors
        );
    }

    pub fn dump_all_neighbors(&self) {
        log_file!(
            "opt2.txt",
            "{:?}",
            self.info.as_ref().unwrap().all_neighbors
        );
    }

    // dump to colors
    pub fn dump_tocolor(&self) {
        let p = "opt2.txt";
        log_file!(p, "to colors:");
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
