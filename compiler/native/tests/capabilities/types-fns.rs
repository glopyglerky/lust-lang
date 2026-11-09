// capability: types: closures, fn items, and fn pointers
// capability: types: `impl Trait` as a whole fn argument or return type
// capability: expressions: closures (Fn, FnMut, FnOnce, and move)
// expect: ok

fn add1(x: u32) -> u32 {
    x + 1
}

fn apply(f: impl Fn(u32) -> u32, x: u32) -> u32 {
    f(x)
}

fn evens(n: u32) -> impl Iterator<Item = u32> {
    (0..n).filter(|x| x % 2 == 0)
}

fn call_mut<F: FnMut()>(mut f: F) {
    f();
    f();
}

fn call_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

fn main() {
    let p: fn(u32) -> u32 = add1;
    let k = 10;
    let mut count = 0;
    call_mut(|| count += 1);
    let s = String::from("owned");
    let moved = move || s;
    println!("{} {} {} {} {}", p(1), apply(|x| x * k, 2), evens(7).count(), count, call_once(moved));
}
