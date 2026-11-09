// An `F: FnOnce` instance with F = &mut C or &C calls the closure through
// the reference (core's impl FnOnce for &F / &mut F): the closure is not
// moved out of the reference (lust check accepted, lust run rejected).
fn call<F: FnOnce()>(f: F) {
    f();
}

fn call_ret<F: FnOnce(i32) -> i32>(f: F) -> i32 {
    f(2)
}

fn call_later<F: FnOnce()>(f: F) {
    let g = f;
    g();
}

fn call_opt<F: FnOnce()>(f: Option<F>) {
    if let Some(f) = f {
        f();
    }
}

fn call_len<F: FnOnce() -> usize>(f: F) -> usize {
    f()
}

fn main() {
    let mut n = 0;
    let mut c = || n += 1;
    call(&mut c);
    call(&mut c);
    call_later(&mut c);
    call_opt(Some(&mut c));
    let r = &mut c;
    call(r);
    println!("{}", n);
    let mut t = 0;
    let mut add = |x: i32| {
        t += x;
        t
    };
    println!("{}", call_ret(&mut add));
    println!("{}", t);
    let v = vec![1, 2, 3];
    let len = move || v.len();
    println!("{} {}", call_len(&len), len());
}
