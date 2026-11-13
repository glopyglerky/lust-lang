// Two partial lines that pass the 1024-byte buffer: the first flushes
// when the second does not fit, the second is lost.
// Abort mode only (unwinding flushes the tail): panics with attempt to add with overflow
fn main() {
    let n = std::env::args().count() as u8;
    print!("{}", "a".repeat(700));
    print!("{}", "b".repeat(700));
    println!("{}", 255 + n);
}
