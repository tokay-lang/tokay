use std::fs::File;
use std::io::{self, Read};
use tokay;

fn main() -> io::Result<()> {
    let program: tokay::vm::Program = {
        let mut file = File::open("../program.json")?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        /*
        let (value, _): (tokay::vm::Program, usize) = bincode::serde::decode_from_slice(&buffer, bincode::config::standard())
            .expect("Decoding failed");
            value
        */

        serde_json::from_slice(&buffer)?
    };

    println!("{:?}", program);
    println!("run...{:?}", program.run_from_str("1 + 2 * 3 + 4"));

    Ok(())
}
