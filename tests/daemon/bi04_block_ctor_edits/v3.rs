fn f() -> u32 {
    #[derive(Debug)]
    struct P(u32);
    let a = P(4).0;
    {
        fn P(x: u32) -> u32 { x * 100 }
        a + P(2)
    }
}
fn main() { println!("{}", f()); }
