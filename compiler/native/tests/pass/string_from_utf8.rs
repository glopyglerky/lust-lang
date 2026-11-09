// String::from_utf8 and str::from_utf8: Rust's valid_up_to and error_len
// for each kind of bad sequence, the errors' Debug and Display (no
// padding), and their methods.
fn show(bytes: Vec<u8>) {
    match String::from_utf8(bytes) {
        Ok(s) => println!("ok {:?}", s),
        Err(e) => {
            let u = e.utf8_error();
            println!("{} | {:?} | {} {:?} | {:?}", e, e, u.valid_up_to(), u.error_len(), e.as_bytes());
            println!("{:?}", e.into_bytes());
        }
    }
}
fn main() {
    show(vec![104, 105]);
    show(vec![104, 0xff, 105]);
    show(vec![0xe2, 0x82]);
    show(vec![0xe2, 0x82, 0x41]);
    show(vec![0xf0, 0x9f, 0x98, 0x80, 0xc0, 0x80]);
    show(vec![0xed, 0xa0, 0x80]);
    show(vec![0xf4, 0x90, 0x80, 0x80]);
    show(vec![0xf0, 0x9f, 0x98]);
    show(vec![0x61, 0xe0, 0x80, 0x80]);
    let a = [0x68, 0x69];
    let s = std::str::from_utf8(&a);
    println!("{:?}", s);
    let b = [0x68, 0xff];
    let t = str::from_utf8(&b);
    println!("{:?}", t);
    match t {
        Ok(_) => {}
        Err(e) => println!("{} {} [{:>40}]", e, e == e.clone(), e),
    }
}
