mod util { pub fn k() -> u32 { 1 } }
fn site() -> u32 {
    struct Pt { x: u8 }
    impl Pt { fn get(&self) -> u32 { self.x } }
    let p = Pt { x: 3 };
    p.get() + util::k()
}
fn main() { println!("{}", site()); }
