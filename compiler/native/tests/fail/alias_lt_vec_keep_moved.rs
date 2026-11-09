// expect: L0311
// rustc E0505: `v` keeps `&s` through `R<'a>` when `s` moves.

type R<'a> = &'a String;
fn keep<'a>(v: &mut Vec<R<'a>>, x: R<'a>) { v.push(x); }
fn main() {
    let mut v = Vec::new();
    let s = String::from("x");
    keep(&mut v, &s);
    drop(s);
    println!("{}", v.len());
}
