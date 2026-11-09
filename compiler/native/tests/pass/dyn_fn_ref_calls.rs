// `&dyn Fn` values from closures (ARCH 2.8, 2.9): the closure signature
// comes from the dyn type, captures by ref and by move, `&str` inputs,
// a generic fn taking `&dyn Fn(T) -> T`, recursion, and `+ Send`.
fn call(f: &dyn Fn(i32) -> i32) -> i32 {
    f(3) + f(4)
}
fn run(f: &dyn Fn()) {
    f();
    f();
}
fn apply(f: &dyn Fn(&str) -> usize) -> usize {
    f("abcd")
}
fn show(f: &dyn Fn(u8) -> String) -> String {
    f(7)
}
fn twice<T>(f: &dyn Fn(T) -> T, x: T) -> T {
    f(f(x))
}
fn fact(n: u64, f: &dyn Fn(u64) -> u64) -> u64 {
    if n == 0 { f(1) } else { n * fact(n - 1, f) }
}
fn main() {
    println!("{}", call(&|n| n * 2));
    let k = 5;
    println!("{}", call(&|n| n + k));
    let s = String::from("hello");
    run(&|| println!("hi {}", s));
    println!("{}", apply(&|t| t.len() * 2));
    println!("{}", show(&|b| format!("<{}>", b)));
    let owned = String::from("xyz");
    let f = move |n: i32| owned.len() as i32 + n;
    println!("{} {}", call(&f), f(1));
    println!("{}", twice(&|s: String| s + "!", String::from("a")));
    println!("{}", fact(5, &|x| x));
    let send: &(dyn Fn() -> u8 + Send) = &|| 4;
    println!("{}", send());
}
