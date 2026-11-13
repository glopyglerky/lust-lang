// expect: L0602
// Valid Rust. lust has no unions (this was a syntax error before).
fn main() {
    union U {
        a: u32,
    }
    let u = U { a: 1 };
    let _ = u;
}
