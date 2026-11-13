// expect: L0612
// rustc E0277: u8 has no From<u32>, known only after the call.
fn main() {
    let mut v = Vec::new();
    let mut first = true;
    loop {
        if !first { let x = u8::from(v.pop().unwrap()); println!("{}", x); break; }
        first = false;
        v.push(250u32);
    }
}
