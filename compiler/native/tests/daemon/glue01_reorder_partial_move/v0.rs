struct S {
    a: i32,
    b: String,
}
fn main() {
    let p = S { a: 3, b: String::from("bee") };
    println!("{} {}", p.b.len(), p.a);
}
