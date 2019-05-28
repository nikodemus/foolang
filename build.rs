use lalrpop;

fn main() {
    lalrpop::process_root().expect("Parser generation failed!");
}
