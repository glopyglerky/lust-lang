// expect: L0306
// rustc E0597: `Pair<'a, T>` ties the result to `b`, which dies first.
type Pair<'a, T> = (&'a T, &'a T);
fn first<'a, T>(p: Pair<'a, T>) -> &'a T { p.0 }
fn main() {
    let a = 1;
    let r;
    {
        let b = 2;
        r = first((&a, &b));
    }
    println!("{}", r);
}
