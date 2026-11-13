mod util { pub fn k() -> u32 { 1 } }
#[derive(Debug, Default)]
struct Shared { a: u8 }
impl Shared { fn val(&self) -> u32 { self.a as u32 + 100 } }
fn outer() -> u32 { Shared::default().val() }
fn site(n: u32) -> u32 {
    let s = Shared { a: n as u8 };
    s.val() + util::k()
}
fn main() { println!("{} {}", site(2), outer()); }
