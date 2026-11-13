// Calls through Fn, FnMut, and FnOnce bounds, monomorphized per closure.
fn call_twice<F: FnMut() -> u32>(mut f: F) -> u32 {
    f() + f()
}
fn call_ref<F: Fn(u32) -> u32>(f: &F, x: u32) -> u32 {
    f(x) * 10
}
fn consume<F: FnOnce() -> String>(f: F) -> String {
    f()
}
fn apply<T, U, F: Fn(T) -> U>(f: F, t: T) -> U {
    f(t)
}
fn main() {
    let mut n = 0u32;
    println!("{}", call_twice(|| {
        n += 1;
        n
    }));
    println!("n={}", n);
    let base = 4u32;
    let add = |x: u32| x + base;
    println!("{}", call_ref(&add, 1));
    println!("{}", add(2));
    let s = String::from("moved");
    println!("{}", consume(move || s));
    println!("{}", apply(|x| x * 3, 5u8));
    println!("{}", apply(|v: Vec<u8>| v.len(), vec![1, 2, 3]));
}
