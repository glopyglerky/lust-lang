// expect: L0511
fn main() {
    pub(in nowhere) static S: u32 = 1;
    println!("{}", S);
}
