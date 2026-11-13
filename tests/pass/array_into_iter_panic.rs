// panic: mid iteration
// A panic mid-iteration over a by-value array (fx-array-iter): the
// elements yielded before it printed and dropped in order.
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    for d in [D(1), D(2), D(3)] {
        println!("for {}", d.0);
        if d.0 == 2 {
            panic!("mid iteration");
        }
    }
    println!("never");
}
