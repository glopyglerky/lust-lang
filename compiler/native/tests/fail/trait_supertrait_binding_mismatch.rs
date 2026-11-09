// expect: L0110
struct V { v: Vec<i32> }
impl V { fn b(&self) -> impl ExactSizeIterator<Item = i32> { self.v.iter() } }
fn main() { let v = V { v: vec![1] }; println!("{}", v.b().len()); }
