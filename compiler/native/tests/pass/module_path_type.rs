fn get(x: crate::S) -> i64 {
    x.v
}

struct S {
    v: i64,
}

fn main() {
    println!("{}", get(S { v: 3 }));
}
