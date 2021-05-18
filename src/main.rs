use ::tokay::repl::repl;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("Tokay v{}", VERSION);
    repl();
}
