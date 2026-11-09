// `Box::as_ref` on a `Box<dyn Fn>` borrows the box only for the call;
// the box's `'static` object lifetime carries over to the result.
fn make(add: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |n| n + add)
}
fn apply(f: &dyn Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}
fn main() {
    let f = make(5);
    println!("{}", apply(f.as_ref(), 3));
    println!("{}", f(1));
}
