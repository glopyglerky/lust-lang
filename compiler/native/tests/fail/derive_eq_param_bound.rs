// expect: L0112
// rustc E0369: `#[derive(PartialEq)]` bounds `T: PartialEq`.
#[derive(PartialEq)]
struct Id<T> {
    raw: u32,
    f: Option<fn() -> T>,
}
struct NC;
fn main() {
    let a: Id<NC> = Id { raw: 1, f: None };
    println!("{}", a == a);
}
