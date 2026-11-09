// expect: L0110
fn h<const M: usize>(a: [i32; M]) -> usize {
    a.len()
}
fn main() {
    println!("{}", h::<u8>([10]));
}
