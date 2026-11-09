use std::fmt::Debug;
struct Acc { v: i64 }
impl Acc {
    fn add(&mut self, x: i64) -> String { self.v += x as i64; self.v  }
    fn get(&self) -> i64  { self.v }
}
fn show<T: Debug>(t: T) -> String { format!("{:?}", t) }
fn main() { let mut a = Acc { v: 1 }; let r = a.add(2); println!("{} {}", show(r), a.get()); }
