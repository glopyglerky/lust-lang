fn f() -> u32 {
    #[derive(Debug)]
    struct P { v: u32 }
    let mk = P;
    mk(3).0 + P(4).0
}
fn main() { println!("{}", f()); }
