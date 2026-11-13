// expect: L0110
enum E { A = 9223372036854775807, B }
fn main() {
    println!("{}", E::A as i64);
}
