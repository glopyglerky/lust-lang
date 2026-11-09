// A std method panics at its name, not at the receiver (track_caller).
// panic: called `Option::unwrap()` on a `None` value
fn find(v: &[u32], x: u32) -> Option<usize> {
    let mut i = 0;
    while i < v.len() {
        if v[i] == x {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn main() {
    let v = vec![3, 1, 4];
    println!("{}", find(&v, 4).expect("present"));
    let n = std::env::args().count();
    let got = find(&v, 9 + n as u32)
        .unwrap();
    println!("{}", got);
}
