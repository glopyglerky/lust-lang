// expect: L0305
struct G(i32);

fn main() {
    let mut a = [G(0)];
    let src = [(G(1), G(2))];
    (a[0], _) = src[0];
    println!("{}", a[0].0);
}
