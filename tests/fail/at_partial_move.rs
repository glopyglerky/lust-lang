// expect: L0301
fn main() {
    let o = Some("x".to_string());
    match o {
        x @ Some(y) => println!("{} {}", x.unwrap(), y),
        None => {}
    }
}
