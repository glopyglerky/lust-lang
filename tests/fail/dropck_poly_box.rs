// expect: L0730
// rustc E0320: Grow<u8> names Grow<(u8, u8)> through Option<Box<..>>, so its drop check never ends.
struct Grow<T> { v: T, g: Option<Box<Grow<(T, T)>>> }
fn main() { let g: Grow<u8> = Grow { v: 1, g: None }; println!("{}", g.v); }
