// expect: L0301
// rustc E0382: `keep` returns, so `p` is used after the move.
fn keep(_b: Box<u8>) -> u8 {
    1
}
fn a(p: Box<u8>) -> Box<u8> {
    if *p == 0 {
        keep(p);
    }
    p
}
fn main() {
    println!("{}", a(Box::new(1)));
}
