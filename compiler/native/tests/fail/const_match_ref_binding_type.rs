// expect: L0110
// rustc E0308: `x` binds by reference through `&E`, so it is a `&u32`.
enum E {
    A(u32),
    B,
}
const X: u32 = match &E::A(5) {
    E::A(x) => {
        let y: u32 = x;
        y
    }
    E::B => 0,
};
fn main() {
    println!("{}", X);
}
