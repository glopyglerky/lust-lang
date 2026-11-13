// JS-number oracle for the float fuzz gate: ryu-js 1.0.2 Buffer::format,
// the call ts-rust's jsnum.rs Display makes. fuzz.sh compiles the crate
// from the local cargo registry sources as an rlib and links this file
// against it; when the crate is absent the JS fuzz is skipped (the C
// harness edge tests still cover directed JS cases).
use std::io::{BufRead, BufWriter, Write};

fn main() {
    let mode = std::env::args().nth(1).expect("mode argument");
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let stdout = std::io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    let mut buf = ryu_js::Buffer::new();
    let mut line = String::new();
    loop {
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let line = line.trim_end_matches('\n');
        match mode.as_str() {
            "js64" => {
                let v = f64::from_bits(u64::from_str_radix(line, 16).unwrap());
                writeln!(out, "{}", buf.format(v)).unwrap();
            }
            "js32" => {
                let v = f32::from_bits(u32::from_str_radix(line, 16).unwrap());
                writeln!(out, "{}", buf.format(v)).unwrap();
            }
            other => {
                eprintln!("bad mode {other}");
                std::process::exit(2);
            }
        }
    }
}
