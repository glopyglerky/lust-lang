// panic: closure body
// A closure whose body only diverges gets the `!` return type (edition
// 2024 fallback); calling it as a statement runs it. Lust used to drop
// the call.
fn main() {
    let ok = || println!("plain");
    ok();
    let f = || {
        println!("in closure");
        panic!("closure body");
    };
    f();
    println!("never");
}
