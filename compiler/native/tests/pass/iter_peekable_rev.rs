// Peekable is double-ended: rev and nth from the back see the
// peeked item only after the inner iterator's back is spent.
fn cp(o: Option<&i32>) -> Option<i32> {
    match o {
        Some(x) => Some(*x),
        None => None,
    }
}
fn main() {
    let v = vec![1, 2, 3];
    let w: Vec<i32> = v.iter().copied().peekable().rev().collect();
    println!("{:?}", w);
    // A peeked item stays at the front when the back is read.
    let mut p = (0..5).peekable();
    let a = cp(p.peek());
    let w: Vec<i32> = p.rev().collect();
    println!("{:?} {:?}", a, w);
    // One peeked element is both front and back.
    let mut p = (0..1).peekable();
    let a = cp(p.peek());
    println!("{:?} {:?}", a, p.rev().collect::<Vec<_>>());
    // Owned items: peek then next must drop each item once.
    let vs: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let mut p = vs.into_iter().peekable();
    println!("{:?}", p.peek());
    println!("{:?} {:?} {:?}", p.next(), p.nth(1), p.next());
    // nth and count after a peek.
    let mut p = (0..10).peekable();
    let a = cp(p.peek());
    println!("{:?} {:?} {:?}", a, p.nth(2), p.count());
}
