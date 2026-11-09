type Two<T, U = u8> = (T, U);
fn first(t: Two<u16>) -> u16 {
    t.0
}
fn main() {
    let t: Two<u16> = (1, 300);
    println!("{} {}", first(t), t.1);
}
