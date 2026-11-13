mod a;
fn get<T>(s: &a::S, t: T) -> T {
    let _ = s.x;
    t
}
fn main() {
    let s = a::S::new();
    println!("{}", get(&s, 3));
}
