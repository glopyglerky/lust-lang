// expect: L0115
fn main() {
    let x: f64 = -1.5;
    println!("{}", match x { ..0.0 => 1, 0.0.. => 2 });
}
