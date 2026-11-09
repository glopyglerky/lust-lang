// Box in generic fns and a generic struct field.
fn wrap<T>(x: T) -> Box<T> { Box::new(x) }
fn unbox<T>(b: Box<T>) -> T { *b }
struct Holder<T> { v: Box<T> }
fn main() {
    let a = wrap(3);
    println!("{}", unbox(a));
    let h = Holder { v: wrap("z".to_string()) };
    println!("{}", h.v);
    let s = unbox(h.v);
    println!("{}", s);
}
