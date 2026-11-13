fn g(0..=255: u8) {}
fn main() {
    let x @ 0..=255 = 3u8;
    println!("{}", x);
    let y @ i8::MIN.. = 4i8;
    println!("{}", y);
    for z @ 0.. in [5u32] {
        println!("{}", z);
    }
    g(1);
    let b @ ..=255 = 4u8;
    println!("{}", b);
    let d @ (i8::MIN..=i8::MAX) = -5i8;
    println!("{}", d);
    let f @ ('\0'..=char::MAX) = 'z';
    println!("{}", f);
    let (true | false) = true;
    println!("or");
    for (Some(true | false) | None) in [Some(false)] {
        println!("elem");
    }
    let v = [7u8];
    let [w @ 0..=255] = v;
    println!("{}", w);
}
