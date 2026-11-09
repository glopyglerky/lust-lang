// expect: L0305
struct D(u8);

fn main() {
    let a = [Some(D(1)), None];
    if let Some(x) = a[0] {
        println!("x {}", x.0);
    }
}
