// expect: L0630
const S: &[u8] = b"ab";
const N: Option<&u8> = S.get(1);
fn main() {
    println!("{:?}", N);
}
