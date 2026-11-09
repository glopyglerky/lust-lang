// Stdout is a LineWriter: a panic=abort loses the partial last line
// (here "tail"), but keeps every complete line and every write of
// 1024 bytes or more.
// Abort mode only (unwinding flushes the tail): panics with attempt to subtract with overflow
fn main() {
    let n = std::env::args().count() as u32;
    print!("{}", "x".repeat(1100));
    println!();
    print!("first ");
    println!("line");
    print!("tail");
    eprintln!("to stderr");
    println!("{}", n - 2);
}
