fn f(x: &(&[bool], u8)) -> usize {
    x.0.len()
}
fn g(x: Option<(u8, &[bool])>) -> usize {
    x.map_or(0, |p| p.1.len())
}
struct W<'a>(&'a [u8], u8);
fn h(w: W) -> usize {
    w.0.len()
}
fn main() {
    println!("{}", f(&(&[true, false], 1)));
    println!("{}", g(Some((1, &[true]))));
    let t: &(u8, &[bool]) = &(1, &[true, true]);
    println!("{}", t.1.len());
    let a1 = [1u8, 2];
    let a2 = [3u8];
    let v: Vec<&[u8]> = vec![&a1, &a2];
    println!("{} {}", v[0].len(), v[1].len());
    let a: [&[u8]; 2] = [&[1, 2, 3], &[4]];
    println!("{} {}", a[0].len(), a[1].len());
    println!("{}", h(W(&[9, 9], 5)));
    let p: (u8, &[bool]) = (2, &[false]);
    println!("{}", p.1.len());
}
