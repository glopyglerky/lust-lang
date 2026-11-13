// A nested fn sees the items of every block around it, and its own
// block's items shadow module items, in the root and in modules.
fn g() -> i32 { 1 }
mod w {
    fn inner(y: i64) -> i64 { y - 1000 }
    fn helper() -> i64 { 5 }
    fn host(x: i64) -> i64 {
        fn inner(y: i64) -> i64 { y * 3 }
        fn n() -> i64 { helper() }
        inner(x) + n()
    }
    pub fn run() { println!("{}", host(3)); }
}
mod a { pub fn g() -> i32 { 10 } }
fn main() {
    w::run();
    fn g() -> i32 { 2 }
    const K: i32 = 7;
    {
        fn b() -> i32 { g() + K }
        println!("{}", b());
        {
            use a::g;
            fn c() -> i32 { g() }
            println!("{}", c());
        }
    }
}
