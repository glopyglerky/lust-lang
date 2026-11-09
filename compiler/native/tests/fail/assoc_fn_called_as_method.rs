// expect: L0104
// rustc E0599: `get` is an associated function (no `self`) and no other `get` is reachable.
struct W(u8);
impl W {
    fn get(this: &W) -> u8 {
        this.0
    }
}
fn main() {
    let w = W(1);
    println!("{}", w.get());
}
