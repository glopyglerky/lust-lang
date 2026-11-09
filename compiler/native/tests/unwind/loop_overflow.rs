// Overflow in a callee inside a loop: the loop body's local and the
// outer locals (a Vec and a tuple of droppable values) drop on unwind.
struct G(&'static str);
impl Drop for G { fn drop(&mut self) { println!("drop {}", self.0); } }
fn ovf(a: u8, b: u8) -> u8 { let _g = G("ovf"); a + b }
fn main() {
    let _x = G("x");
    let v: Vec<G> = vec![G("v1"), G("v2")];
    let t = (G("t1"), 5);
    let mut total = 0u8;
    for i in 0..10u8 {
        let _loop = G("loop");
        total = ovf(total, 100 + i);
        print!("{} ", total);
    }
    drop(v);
    drop(t);
}
