use std::collections::{HashMap, HashSet};

use crate::{backend::operand::Reg, log_file};

pub fn dump_interefgraph(ig: &HashMap<Reg, HashSet<Reg>>, path: &str) {
    for (r, nbs) in ig.iter() {
        log_file!(path, "node {}", r);
        nbs.iter().for_each(|nb| {
            log_file!(path, "({},{})", r, nb);
        });
    }
}
