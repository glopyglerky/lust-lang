// expect: L0301
// rustc E0382: an inline const block is borrow-checked as a body of its own.
struct S {
    x: u32,
}
fn main() {
    let c = const {
        let s = S { x: 1 };
        let a = s;
        let b = s;
        a.x + b.x
    };
    println!("{}", c);
}
