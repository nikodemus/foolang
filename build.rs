use lalrpop;

fn main() {
    lalrpop::process_root().expect("LALRPOP parser generation failed!");
}
