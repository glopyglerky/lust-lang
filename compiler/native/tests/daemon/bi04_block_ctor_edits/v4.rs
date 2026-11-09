fn f() -> u32 {
    #[derive(Debug)]
    struct P;
    let a = { let _p = P; 1 };
    {
        #[derive(Debug)]
        struct P(u32);
        a + P(2).0
    }
}
fn main() { println!("{}", f()); }
