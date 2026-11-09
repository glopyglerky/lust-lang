struct S {
    x: u32,
}
const fn f(s: &S) -> u32 {
    s.x
}
const C: u32 = {
    let s = S { x: 3 };
    f(&s) + f(&s)
};
fn main() {
    println!("{}", C);
}
