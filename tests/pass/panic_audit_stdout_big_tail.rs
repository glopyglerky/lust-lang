// A print whose tail after the last newline is 1024 bytes or more goes
// straight to fd 1 (BufWriter), so it survives the abort.
// panic: index out of bounds: the len is 0 but the index is 1
fn main() {
    let n = std::env::args().count();
    print!("a\n{}", "y".repeat(1500));
    print!("held");
    let v: Vec<u8> = Vec::new();
    println!("{}", v[n]);
}
