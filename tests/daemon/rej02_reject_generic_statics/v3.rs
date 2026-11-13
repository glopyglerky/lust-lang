// Collect rejects in a generic program with nested statics and a const fn:
// generic reuse keeps the entries made before the failed collect and drops
// the ones it stored (gen_reuse::drop_env). v2 rejects and adds a nested
// static (static ids move); v3 fixes it; v4 goes back to v1; v5 is a body
// error in a generic fn (a template error: the env stays); v6 fixes it; v7
// rejects with a const fn edit; v8 goes back to v1. Lines above the edit
// points stay the same in every version.
const fn base() -> u32 {
    3
}
const K: u32 = base();
struct W<T> {
    x: T,
}
impl<T: Copy + std::fmt::Debug> W<T> {
    fn get(&self) -> T {
        self.x
    }
}
fn show<T: std::fmt::Debug>(t: T) -> String {
    let _e: u32 = 2;
    format!("{:?}", t)
}
fn counter() -> u32 {
    static N: u32 = 5;
    N + K
}
fn main() {
    let w = W { x: 4u8 };
    println!("{} {} {} {}", show(w.get()), counter(), show("a"), counter2());
}
fn counter2() -> u32 {
    static M: u32 = 9;
    M
}
