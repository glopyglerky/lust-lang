// expect: L0122
fn main() {
    let o = &Some(1);
    match o {
        Some(ref x) => println!("{}", x),
        None => {}
    }
}
