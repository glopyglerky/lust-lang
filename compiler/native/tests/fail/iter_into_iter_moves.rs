// expect: L0301
fn main() {
    let v = vec![String::from("x")];
    let w: Vec<String> = v.into_iter().collect();
    println!("{:?} {:?}", v, w);
}
