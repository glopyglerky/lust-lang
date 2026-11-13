// str::split_at and split_at_checked: both parts borrow the receiver; the
// checked form gives None past the end or inside a char.
fn main() {
    let (kind, name) = "dfoo".split_at(1);
    println!("{kind} {name}");
    let s = String::from("héllo");
    let (a, b) = s.split_at(3);
    println!("[{a}|{b}]");
    for mid in 0..8 {
        match s.split_at_checked(mid) {
            Some((x, y)) => println!("{mid}: {x}|{y}"),
            None => println!("{mid}: none"),
        }
    }
    let boxed: Box<str> = "x😀y".into();
    println!("{:?} {:?}", boxed.split_at_checked(2), boxed.split_at(5));
}
