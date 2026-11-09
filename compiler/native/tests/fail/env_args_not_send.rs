// expect: L0612
// rustc E0277: std::env::Args is not Send.
fn main() {
    let a = std::env::args();
    std::thread::spawn(move || { for x in a { println!("{}", x); } }).join().unwrap();
}
