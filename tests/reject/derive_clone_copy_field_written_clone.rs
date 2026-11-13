// expect: L0112
// Valid Rust: a Copy type with a written Clone is copied in some places
// and cloned through its impl in others; derived glue over it is not
// supported yet.
#[derive(Debug, Copy)]
struct C(u8);
impl Clone for C { fn clone(&self) -> C { println!("clone C"); C(self.0 + 1) } }
#[derive(Debug, Clone)]
struct NonCopy { c: C, s: String }
#[derive(Debug, Clone, Copy)]
struct AllCopy { c: C, k: u8 }
fn main() {
    let n = NonCopy { c: C(1), s: String::new() };
    println!("{:?}", n.clone());
    let a = AllCopy { c: C(5), k: 1 };
    println!("{:?}", a.clone());
    let t = (C(9), String::from("t"));
    println!("{:?}", t.clone());
}
