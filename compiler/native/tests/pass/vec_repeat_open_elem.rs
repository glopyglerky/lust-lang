// vec![x; n] whose element type a later use fixes: the Clone check waits for body end.
fn main() {
    let mut d = vec![Vec::new(); 3];
    d[1].push(7u8);
    let mut o = vec![None; 2];
    o[0] = Some(String::from("x"));
    println!("{:?} {:?}", d, o);
}
