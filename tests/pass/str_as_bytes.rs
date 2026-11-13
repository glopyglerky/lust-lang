// `str::as_bytes` (W-STD-B): the `&[u8]` view of a str, borrowing it.
fn first_byte(s: &str) -> Option<u8> {
    match s.as_bytes().first() {
        Some(b) => Some(*b),
        None => None,
    }
}
fn main() {
    let s = "héllo";
    let b = s.as_bytes();
    println!("{} {:?} {:?}", b.len(), &b[..3], first_byte("z"));
    let owned = String::from("ab");
    let t: &str = &owned;
    println!("{:?} {:?}", t.as_bytes(), owned.as_bytes());
    let r: &[u8];
    {
        let k = String::from("q");
        r = k.as_str().as_bytes();
        println!("{r:?}");
    }
}
