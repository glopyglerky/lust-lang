// Box<&C>, Box<&Box<dyn Fn>>, and &&C satisfy Fn-family bounds and call C by Fn.
fn call<F: FnOnce() -> i32>(f: F) -> i32 { f() }
fn call_mut<F: FnMut() -> i32>(mut f: F) -> i32 { f() }
fn call_fn<F: Fn() -> i32>(f: F) -> i32 { f() }
fn main() {
    let c = || 5;
    println!("{}", call(Box::new(&c)));
    println!("{}", call(&c));
    println!("{}", call(Box::new(c)));
    println!("{}", call_mut(Box::new(&c)));
    println!("{}", call_fn(Box::new(&c)));
    println!("{}", call(&&c));
    let b: Box<dyn Fn() -> i32> = Box::new(c);
    println!("{}", call(&b));
    println!("{}", call(Box::new(&b)));
}
