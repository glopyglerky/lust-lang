// `dbg!` prints with `{:#?}`, so composite values span lines on stderr.
#[derive(Debug)]
struct S {
    a: i32,
    b: Vec<i32>,
}

fn main() {
    let v = dbg!(Some(vec![1, 2]));
    let s0 = S { a: 1, b: vec![3] };
    let s = dbg!(s0);
    let t = dbg!((v.map(|v| v.len()).unwrap(), s.a, s.b.len()));
    println!("{} {} {}", t.0, t.1, t.2);
}
