fn main() {
    match left::schedule(4) {
        Ok(q) => println!("{:?} total {:?}", q, q.iter().fold(std::time::Duration::ZERO, |a, &b| a + b)),
        Err(e) => println!("error {e}"),
    }
    match left::schedule(9) {
        Ok(q) => println!("{q:?}"),
        Err(e) => println!("error {e}"),
    }
    let step = base_step();
    println!("{:?} {:?} {}", step, right::words(), right::meters::<u8>(2.5).0);
    println!("{:?} {:?}", right::first_long(&["ab", "abcd", "abcdef"], 3), right::first_long(&["ab"], 3));
}

fn base_step() -> std::time::Duration {
    left::schedule(2).map(|q| q[1]).unwrap_or_default()
}
