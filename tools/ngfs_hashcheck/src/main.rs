use std::{env, fs};
fn main() {
    let p = env::args().nth(1).expect("path");
    let exp = env::args().nth(2).expect("expected");
    let data = fs::read(&p).expect("read");
    let got = format!("{:x}", blake3::hash(&data));
    if got == exp {
        println!("OK");
        std::process::exit(0);
    }
    eprintln!("MISMATCH:{got}");
    std::process::exit(2);
}
