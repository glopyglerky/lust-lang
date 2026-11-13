// An index panic three frames deep: each frame drops its live locals,
// innermost first, then main exits 101.
struct G(i32);
impl Drop for G { fn drop(&mut self) { println!("drop {}", self.0); } }
fn leaf(v: &Vec<i32>, i: usize) -> i32 {
    let _g = G(3);
    let s = String::from("tmp");
    println!("leaf {}", s);
    v[i] + 1
}
fn mid(n: i32) -> i32 {
    let _a = G(1);
    let v = vec![n, n + 1];
    let _b = G(2);
    let x = leaf(&v, 7);
    println!("never {}", x);
    x
}
fn main() {
    let _m = G(0);
    println!("start");
    let r = mid(4);
    println!("end {}", r);
}
