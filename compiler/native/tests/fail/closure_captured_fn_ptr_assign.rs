// expect: L0308
// rustc E0594: `cb` is captured but not declared `mut`.
fn main() {
    let cb: fn(u32) -> u32 = |x| x;
    let c = || {
        for _ in 0..2 {
            cb = |x| x + 1;
        }
    };
    c();
}
