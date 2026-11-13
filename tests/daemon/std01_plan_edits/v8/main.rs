mod a;

fn main() {
    let v: Option<u8> = None;
    println!("{}", v.is_none());
    a::go();
}
