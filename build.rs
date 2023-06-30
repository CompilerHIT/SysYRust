extern crate lalrpop;

fn main() {
    lalrpop::process_root().unwrap();
    println!("cargo:rustc-link-search=./deps");
}
