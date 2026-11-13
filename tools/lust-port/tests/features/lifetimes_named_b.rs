// feature: lifetimes.named
// form: named lifetimes in fns, a lifetime enum, and an outlives bound
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() >= y.len() { x } else { y } }
enum Tok<'a> { Word(&'a str), Eof }
fn first<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str where 'a: 'b { x }
fn main() {
    let t = Tok::Word("hi");
    let w = match t { Tok::Word(s) => s, Tok::Eof => "" };
    println!("{} {} {}", longest("ab", "cde"), w, first("xy", "z"));
}
