struct S {
    b: String,
    a: i32,
}
fn main() {
    let p = S { a: 3, b: String::from("bee") };
    let moved = p.b;
    println!("{} {}", moved.len(), p.a);
}
