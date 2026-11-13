// expect: L0118
fn main() {
    let x: f32 = 1.0;
    println!("{}", matches!(x, 1e400));
}
