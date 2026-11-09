// expect: L0610
// rustc E0282: nothing fixes the constructor value's type.
struct W<T>(T);
fn main() {
    let f = W;
    let _ = f;
}
