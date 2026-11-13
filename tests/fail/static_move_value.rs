// expect: L0305
// rustc E0507: a non-Copy static's value cannot move out (it was copied).
struct D(u8);
static S: D = D(1);
fn main() {
    let x = S;
    println!("{}", x.0);
}
