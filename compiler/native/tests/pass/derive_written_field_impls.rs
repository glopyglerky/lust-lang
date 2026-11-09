// Derived Default and Clone over fields whose impls are written by hand:
// the derived bodies call the field type's impl (once per field, in field
// order; per element for tuples, arrays, Box, Vec), as rustc's do.
#[derive(Debug)]
struct Cfg { level: u8, name: String }
impl Default for Cfg { fn default() -> Cfg { println!("Cfg::default"); Cfg { level: 3, name: String::from("x") } } }
#[derive(Debug, Default)]
struct App { cfg: Cfg, n: u32, pair: (Cfg, u8), arr: [Cfg; 2], b: Box<Cfg>, o: Option<Cfg> }
#[derive(Debug)]
struct Big { a: [u64; 8], s: String }
impl Default for Big { fn default() -> Big { Big { a: [7; 8], s: String::from("big") } } }
#[derive(Debug, Default)]
struct W { b: Big, x: u8 }
#[derive(Debug)]
struct Handle(u32);
impl Clone for Handle { fn clone(&self) -> Handle { println!("clone {}", self.0); Handle(self.0 + 100) } }
#[derive(Debug, Clone)]
struct Owner { h: Handle, v: Vec<Handle>, t: (Handle, String) }
fn main() {
    let a = App::default();
    println!("{:?}", a);
    let b = App { n: 7, ..Default::default() };
    println!("{} {}", b.n, b.cfg.level);
    println!("{:?}", W::default());
    let mut taken = App::default();
    let old = std::mem::take(&mut taken.cfg);
    println!("{} {}", old.level, taken.cfg.name);
    let o = Owner { h: Handle(1), v: vec![Handle(2), Handle(3)], t: (Handle(4), String::from("s")) };
    let p = o.clone();
    println!("{:?}", p);
}
