#[repr(C)]
struct S {
    v: i64,
}

fn main() {
    let s = S { v: 1 };
    println!("{}", s.v);
}
