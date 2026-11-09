// A user type named Utf8Error does not hide std's for from_utf8.
struct Utf8Error(u8);
fn main() {
    let u = Utf8Error(3);
    let r = String::from_utf8(vec![104]);
    println!("{} {:?}", u.0, r);
}
