// A char step past char::MAX is the trait default's `expect`.
fn main() {
    let mut r = '\u{10FFFE}'..;
    println!("{:?}", r.next());
    r.next();
}
