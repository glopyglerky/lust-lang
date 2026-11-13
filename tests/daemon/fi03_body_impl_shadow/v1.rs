struct Restore(u8);
impl Drop for Restore { fn drop(&mut self) { println!("module restore {}", self.0); } }
impl Restore { fn get(&self) -> u8 { self.0 } }
fn f() -> u8 {
    struct Restore(u16);
    impl Drop for Restore { fn drop(&mut self) { println!("block restore {}", self.0); } }
    impl Restore { fn get(&self) -> u8 { self.0 as u8 + 100 } }
    let r = Restore(1);
    r.get()
}
fn main() { println!("{}", f()); let m = Restore(2); println!("{}", m.get()); }
