// expect: L0104
// rustc E0599: `#[derive(Clone)]` bounds `T: Clone`; `NC` is not Clone.
#[derive(Clone)]
struct Id<T> {
    raw: u32,
    f: Option<fn() -> T>,
}
struct NC;
fn main() {
    let a: Id<NC> = Id { raw: 1, f: None };
    let b = a.clone();
    println!("{} {}", a.raw, b.raw);
}
