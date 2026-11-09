mod util { pub fn k() -> u32 { 1 } }
fn site(n: u32) -> u32 {
    let a = n + 1;
    let _b = a * 2;
    fn helper(v: u32) -> u32 { v * 3 }
    struct W(u32);
    impl W { fn get(&self) -> u32 { self.0 + 2 } }
    let w = W(a);
    helper(a) + w.get() + util::k()
}
fn main() { println!("{}", site(1)); }
