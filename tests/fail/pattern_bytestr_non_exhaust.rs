// expect: L0115
fn main() {
    let s: &[u8; 2] = b"ab";
    println!("{}", match s { b"ab" => 1 });
}
