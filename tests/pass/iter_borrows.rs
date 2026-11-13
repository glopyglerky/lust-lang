// Borrows through iterators: items borrow the collection, not the
// iterator, so they outlive it; `iter_mut` items are disjoint.
fn main() {
    let v = vec![String::from("a"), String::from("b"), String::from("c")];
    let first = {
        let mut it = v.iter();
        it.next()
    };
    let mut it = v.iter();
    let x = it.next();
    let y = it.next();
    drop(it);
    println!("{:?} {:?} {:?}", first, x, y);
    let long = v.iter().find(|s| s.len() > 0);
    println!("{:?}", long);
    let mut w = vec![1, 2, 3];
    let mut im = w.iter_mut();
    let a = im.next().unwrap();
    let b = im.next().unwrap();
    *a += 10;
    *b += 20;
    println!("{:?}", w);
    let n = w.len();
    for x in w.iter_mut().skip(1) {
        *x += n as i32;
    }
    println!("{:?}", w);
    counted();
}

// A counting closure's capture ends with the iterator: the items
// only borrow the collection.
fn counted() {
    let v = vec![1, 2, 3];
    let mut n = 0;
    let first = v.iter().inspect(|_| n += 1).next();
    n += 1;
    let mut m = 0;
    let all: Vec<&i32> = v.iter().filter(|_| {
        m += 1;
        true
    }).collect();
    m += 1;
    println!("{:?} {} {:?} {}", first, n, all, m);
}
