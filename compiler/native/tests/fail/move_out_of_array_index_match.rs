// expect: L0305
struct D(u8);

fn main() {
    let a = [D(1), D(2)];
    match a[1] {
        x => println!("x {}", x.0),
    }
}
