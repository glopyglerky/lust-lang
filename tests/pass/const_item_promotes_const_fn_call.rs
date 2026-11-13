// In a const or static initializer, rustc promotes a `const fn` call with
// constant arguments, so the borrow lives for 'static.
struct S {
    x: u32,
}
const fn mk() -> S {
    S { x: 4 }
}
const fn idr(s: &S) -> &S {
    s
}
const C: u32 = {
    let r = idr(&mk());
    r.x
};
static R: &S = idr(&mk());
fn main() {
    let a = [0u8; {
        let r = idr(&mk());
        r.x as usize
    }];
    println!("{} {} {}", C, R.x, a.len());
}
