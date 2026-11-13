trait Tr { fn v(&self) -> u32; }
fn show<T: Tr>(t: &T) -> u32 { t.v() + 1 }
fn site() -> u32 {
    const K: u32 = 3;
    struct G(&'static str, u32);
    impl Drop for G {
        fn drop(&mut self) { println!("drop {}", self.0); }
    }
    impl Tr for G {
        fn v(&self) -> u32 { self.1 + K }
    }
    let a = G("a", 1);
    let _b = G("b", 2);
    show(&a)
}
fn main() { println!("{}", site()); }
