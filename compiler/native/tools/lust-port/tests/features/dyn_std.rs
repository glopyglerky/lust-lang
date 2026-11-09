// feature: dyn.std
// form: &mut dyn over a std trait, io::Write into a Vec as in ts-rust emitter writers
use std::io::Write as _;
fn emit(w: &mut dyn std::io::Write) {
    let _ = write!(w, "x{}", 1);
}
fn main() {
    let mut buf: Vec<u8> = Vec::new();
    emit(&mut buf);
    println!("{}", buf.len());
}
