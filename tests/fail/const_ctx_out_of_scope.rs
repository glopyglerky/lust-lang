// expect: L0630
// rustc E0425: a static initializer cannot read another fn's const.
fn a() -> u32 {
    const Q: u32 = 7;
    Q
}

fn main() {
    static B: u32 = Q + 1;
    println!("{} {}", a(), B);
}
