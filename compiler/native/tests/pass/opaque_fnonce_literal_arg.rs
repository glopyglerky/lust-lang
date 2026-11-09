// A generic fn returning `impl FnOnce`/`impl Fn` called with a literal
// argument (`mk(5)`): the literal's type was still a var when the hidden
// closure's instance was mapped, so codegen called the template closure.
// Calls at different type args in one body each get their own closure
// signature (the f64 one printed through the i32 one's output type).
fn mk<T>(t: T) -> impl FnOnce() -> T {
    move || t
}

fn pair<T>(t: T) -> impl FnOnce() -> (T, i32) {
    move || (t, 1)
}

fn count<T>(t: T) -> impl Fn() -> i32 {
    move || {
        let _ = &t;
        7
    }
}

fn with<T>(t: T) -> impl FnOnce(i32) -> (T, i32) {
    move |x| (t, x)
}

fn take<F: FnOnce() -> R, R>(f: F) -> R {
    f()
}

fn main() {
    let h = mk(5);
    println!("{}", h());
    println!("{}", mk(2.5)());
    println!("{:?}", pair([1, 2])());
    println!("{}", take(mk(9)));
    let a = mk(1);
    let b = mk(2);
    println!("{} {}", a(), b());
    println!("{}", count(3)());
    let i = mk(5i32);
    let f = mk(2.5f64);
    let t = mk(true);
    let w = mk(300u64);
    println!("{} {} {} {}", i(), f(), t(), w());
    let x = with('c');
    let y = with(1.5f32);
    println!("{:?} {:?}", x(1), y(2));
}
