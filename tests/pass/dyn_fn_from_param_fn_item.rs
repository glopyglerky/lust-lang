// An Fn-bound type param instantiated at a fn item unsizes to
// `Box<dyn Fn>` in that instance.
struct Bus { hooks: Vec<Box<dyn Fn(u32) -> u32>> }
impl Bus { fn on<F: Fn(u32) -> u32 + 'static>(&mut self, f: F) { self.hooks.push(Box::new(f)); } }
fn double(x: u32) -> u32 { x * 2 }
fn main() { let mut b = Bus { hooks: Vec::new() }; b.on(double); println!("{}", (b.hooks[0])(4)); }
