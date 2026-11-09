// expect: L0612
// rustc E0277: an `FnMut` bound does not give `Fn`.
struct Bus { hooks: Vec<Box<dyn Fn(u32) -> u32>> }
impl Bus { fn on<F: FnMut(u32) -> u32 + 'static>(&mut self, f: F) { self.hooks.push(Box::new(f)); } }
fn main() { let mut b = Bus { hooks: Vec::new() }; b.on(|x| x); println!("{}", b.hooks.len()); }
