// expect: L0110
const K: i32 = 4;
enum E {
    A = K,
}

fn main() {
    let _e = E::A;
    println!("x");
}
