fn get<T>(s: &base::S, t: T) -> i32 {
    let _ = &t;
    s.y
}
fn main() {
    let s = base::S::new();
    println!("{}", get(&s, 3));
}
