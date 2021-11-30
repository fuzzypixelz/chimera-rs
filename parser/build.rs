extern crate lalrpop;

fn main() {
    lalrpop::process_root().unwrap();
    // lalrpop::Configuration::new()
    //     .log_info()
    //     .process_current_dir()
    //     .unwrap();
}
