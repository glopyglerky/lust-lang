struct S {
    a: i64,
    b: i64,
}

fn main() {
    let s1 = S { a: 1, b: 2 };
    let s2 = S { a: 9, ..s1 };
    println!("{}", s2.b);
}
