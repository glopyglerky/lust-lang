// expect: L0301
// rustc E0382: a const initializer block is borrow-checked like a fn body.
struct S {
    x: u32,
}
const C: (S, S) = {
    let s = S { x: 1 };
    (s, s)
};
fn main() {
    println!("{}", C.0.x);
}
