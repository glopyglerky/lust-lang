// expect: L0110
// A nested fn parameter named like a block `use` of a block enum's
// variant is that variant's pattern (rustc E0308 here).
fn main() {
    #[allow(dead_code)]
    enum En { E, Other }
    use En::E;
    fn f(E: u8) -> u8 { E }
    println!("{}", f(2));
}
