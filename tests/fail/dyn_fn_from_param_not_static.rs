// expect: L0705
// rustc E0310: `F` may not live long enough for a `'static` `Box<dyn Fn>`.
struct Bus { hooks: Vec<Box<dyn Fn(u32) -> u32>> }
impl Bus { fn on<F: Fn(u32) -> u32>(&mut self, f: F) { self.hooks.push(Box::new(f)); } }
fn main() { let mut b = Bus { hooks: Vec::new() }; b.on(|x| x); println!("{}", b.hooks.len()); }
