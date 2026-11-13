// A zero repeat moves a local in and drops it at the statement.
struct Dr(u8);
impl Drop for Dr { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() { let x = Dr(5); println!("a"); { let a = [x; 0]; println!("len {}", a.len()); } println!("b"); }
