fn main() {
    let x = 3;
    match &x {
        &v => println!("{}", v),
    }
}
