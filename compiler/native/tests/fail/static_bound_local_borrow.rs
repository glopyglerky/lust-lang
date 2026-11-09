// expect: L0306
// rustc E0597: `T: 'static` with `T = &i32` needs the borrow of `x` to be
// `'static` (check proves the bound's types; the regions are borrowck's).
fn keep<T: 'static>(t: T) -> usize { std::mem::size_of_val(&t) }
fn main() {
    let x = 5;
    let n = keep(&x);
    println!("{n}");
}
