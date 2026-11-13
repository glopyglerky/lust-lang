fn main() {
    let c = true;
    let o: Option<i32> = Some(2);
    let Some(x) = (if c { o } else { None }) else { return };
    println!("{}", x);
    let Some(y) = (match c {
        true => o,
        false => None,
    }) else {
        return;
    };
    println!("{}", y);
    let Some(z) = ({ o }) else { return };
    println!("{}", z);
}
