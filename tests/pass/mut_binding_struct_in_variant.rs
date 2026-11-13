// A &mut binding of a struct field inside an Option, Result, or enum variant
// (borrowck used to read the struct's field under the variant index).
struct P { a: i8, b: bool }
enum E { A(P), B { p: P, q: Option<P> } }
fn main() {
    let mut v = Some(P { a: 1, b: false });
    if let Some(P { b, .. }) = &mut v { *b = true; }
    let mut r: Result<i32, P> = Err(P { a: 2, b: false });
    if let Err(P { b, a }) = &mut r { *b = true; *a += 1; }
    let mut e = E::B { p: P { a: 1, b: false }, q: Some(P { a: 5, b: false }) };
    if let E::B { q: Some(P { a, b }), .. } = &mut e { *a += 1; *b = true; }
    if let E::B { q: Some(x), .. } = e { println!("{} {}", x.a, x.b); }
    let mut o = Some(Some(P { a: 3, b: false }));
    if let Some(Some(P { b, .. })) = &mut o { *b = true; }
    println!("{} {}", v.unwrap().b, o.unwrap().unwrap().b);
    if let Err(p) = r { println!("{} {}", p.a, p.b); }
}
