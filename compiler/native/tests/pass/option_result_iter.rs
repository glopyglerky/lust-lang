// Option and Result iterators (fx-option-rest): iter, iter_mut,
// into_iter, `for` over all six forms, adapters and consumers over
// them, DoubleEnded and ExactSize results, and drop order.

fn exact<I: ExactSizeIterator>(i: I) -> usize {
    i.len()
}

fn from_back<I: DoubleEndedIterator>(mut i: I) -> usize {
    let mut n = 0;
    while i.next_back().is_some() {
        n += 1;
    }
    n
}

fn into_sum<I: IntoIterator<Item = i32>>(i: I) -> i32 {
    i.into_iter().sum()
}

struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    // `for` over the value and reference forms.
    for x in Some(1) {
        println!("{x}");
    }
    for x in None::<i32> {
        println!("never {x}");
    }
    let o: Option<i32> = Some(2);
    for x in &o {
        println!("{x}");
    }
    let mut m: Option<i32> = Some(3);
    for x in &mut m {
        *x += 10;
    }
    println!("{:?}", m);
    let r: Result<i32, String> = Ok(4);
    for x in r {
        println!("{x}");
    }
    let e: Result<i32, String> = Err(String::from("no"));
    for x in &e {
        println!("never {x}");
    }
    let mut mr: Result<i32, String> = Ok(5);
    for x in &mut mr {
        *x *= 2;
    }
    println!("{:?}", mr);

    // Sources with adapters and consumers.
    let a = Some(6);
    println!("{:?}", a.iter().collect::<Vec<_>>());
    println!("{:?}", a.into_iter().chain(Some(7)).chain(Ok::<_, String>(8)).collect::<Vec<_>>());
    let b: Result<i32, String> = Ok(9);
    println!("{:?}", b.iter().map(|x| x + 1).collect::<Vec<_>>());
    println!("{}", b.iter().zip(Some(2).iter()).map(|(x, y)| x * y).sum::<i32>());
    let c: Option<String> = Some(String::from("s"));
    println!("{:?}", c.into_iter().collect::<Vec<_>>());
    let d: Option<i32> = None;
    println!("{:?} {:?} {}", d.iter().next(), d.into_iter().next(), d.iter().rev().count());

    // iter_mut writes through.
    let mut s = Some(String::from("a"));
    for part in s.iter_mut() {
        part.push('b');
    }
    println!("{:?}", s);

    // ExactSize and DoubleEnded, also through adapters and bounds.
    let o2 = Some(10);
    let mut it = o2.iter();
    println!("{} {:?} {:?} {}", it.len(), it.next_back(), it.next(), it.len());
    println!("{:?} {:?}", o2.iter().size_hint(), None::<i32>.iter().size_hint());
    println!("{}", exact(o2.iter().map(|x| *x)));
    println!("{}", from_back(Some(11).into_iter()));
    println!("{}", into_sum(Ok::<_, String>(12)));
    println!("{}", into_sum(Some(13)));
    println!("{}", o2.iter().take(1).len() + o2.iter().skip(1).len());

    // extend and collect through IntoIterator.
    let mut v: Vec<i32> = vec![0];
    v.extend(Some(1));
    v.extend(&Some(2));
    v.extend(Ok::<_, String>(3));
    println!("{:?}", v);
    let flat: Vec<i32> = vec![Some(4), None, Some(5)].into_iter().flatten().collect();
    let flat_r: Vec<i32> = vec![Ok::<_, String>(6), Err(String::from("x"))].into_iter().flatten().collect();
    println!("{:?} {:?}", flat, flat_r);

    // Named iterator types.
    let j: std::option::Iter<'_, i32> = o2.iter();
    println!("{}", j.len());
    let k: std::result::IntoIter<i32> = Ok::<_, String>(14).into_iter();
    println!("{}", k.count());

    // Drop order: an `Err` payload drops at `into_iter`, an unyielded
    // `Some` payload drops with the iterator.
    let it = Err::<i32, D>(D(1)).into_iter();
    println!("mid");
    drop(it);
    let it2 = Some(D(2)).into_iter();
    println!("mid2");
    drop(it2);
    println!("end");
}
