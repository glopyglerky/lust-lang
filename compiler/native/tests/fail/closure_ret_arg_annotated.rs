// expect: L0733
// rustc: lifetime may not live long enough. An annotated `&i32` argument
// is higher-ranked; the result cannot borrow it.
fn main() {
    let f = |x: &i32| x;
    let a = 5;
    println!("{}", f(&a));
}
