// expect: L0115
fn a<T>(o: Option<T>) -> u8 {
    match o {
        None => 1,
    }
}
fn main() {
    println!("{}", a::<u8>(None));
}
