// expect: L0301
// rustc E0382: `#[derive(Copy)]` bounds `T: Copy`, so `Id<String>` moves.
#[derive(Clone, Copy)]
struct Id<T> {
    raw: u32,
    f: Option<fn() -> T>,
}
fn main() {
    let a: Id<String> = Id { raw: 1, f: None };
    let b = a;
    println!("{} {}", a.raw, b.raw);
}
