// A field's drop panics inside struct drop glue: the other fields
// still drop, then the outer local.
struct P(i32);
impl Drop for P { fn drop(&mut self) { println!("drop {}", self.0); if self.0 == 2 { panic!("drop panic {}", self.0); } } }
struct Pair { a: P, b: P, c: P }
fn main() {
    let _x = P(9);
    {
        let _p = Pair { a: P(1), b: P(2), c: P(3) };
        println!("inner");
    }
    println!("after");
}
