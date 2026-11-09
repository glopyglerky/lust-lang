// A zero repeat evaluates its operand once and drops it.
struct Dr(u8);
impl Drop for Dr { fn drop(&mut self) { println!("drop {}", self.0); } }
fn mk(v: u8) -> Dr { println!("mk {}", v); Dr(v) }
fn main() { println!("a"); { let a = [Dr(7); 0]; println!("len {}", a.len()); } { let b = [mk(9); 0]; println!("len {}", b.len()); } println!("b"); }
