struct S {
    b: String,
    a: i32,
}
fn main() {
    let p = S { a: 3, b: String::from("bee") };
    println!("start");
    let f = || p.b.len();
    println!("{} {}", f(), p.a);
}
