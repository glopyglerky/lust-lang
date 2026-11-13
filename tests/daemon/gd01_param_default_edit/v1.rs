struct S<T, U = u8> {
    a: T,
    b: U,
}
fn show(s: &S<i32>) -> bool {
    s.b.checked_add(1).is_none()
}
fn main() {
    let s: S<i32> = S { a: 1, b: 255 };
    println!("{} {}", s.a, show(&s));
}
