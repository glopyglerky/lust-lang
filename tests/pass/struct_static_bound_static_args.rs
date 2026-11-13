// A struct's `T: 'static` bound holds for a literal, a `&'static`
// global, and an owned value.
struct S<T: 'static> { t: T }
static G: u32 = 7;
fn main() {
    let a = S { t: "lit" };
    let b = S { t: &G };
    let c = S { t: String::from("own") };
    println!("{} {} {}", a.t, b.t, c.t);
}
