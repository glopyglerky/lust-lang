// expect: L0306
// rustc E0716: an inline const block promotes no `const fn` call (a const item does).
struct S {
    x: u32,
}
const fn mk() -> S {
    S { x: 4 }
}
const fn idr(s: &S) -> &S {
    s
}
fn main() {
    let c = const {
        let r = idr(&mk());
        r.x
    };
    println!("{}", c);
}
