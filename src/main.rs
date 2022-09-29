use std::process;

fn main() {
    if let Err(err) = symbolic::run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}
