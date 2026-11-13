// Drop order of `array::IntoIter` for non-Copy elements
// (fx-array-iter): partial consumption from either end, early
// break, early return, and adapters that own the iterator. What
// was not yielded drops front to back, exactly once.
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn early() {
    let mut it = [D(31), D(32), D(33)].into_iter();
    let _a = it.next();
    println!("early in");
    return;
}

fn main() {
    // Partial consume from the front, then drop the iterator.
    let mut it = [D(1), D(2), D(3), D(4), D(5)].into_iter();
    let a = it.next().unwrap();
    println!("got {}", a.0);
    drop(a);
    println!("-- drop it");
    drop(it);
    println!("-- done1");
    // Partial consume from both ends.
    let mut it = [D(1), D(2), D(3), D(4), D(5)].into_iter();
    let b = it.next_back().unwrap();
    println!("gotb {}", b.0);
    drop(b);
    let c = it.next().unwrap();
    println!("gotc {}", c.0);
    drop(c);
    println!("-- drop it2");
    drop(it);
    println!("-- done2");
    // Early break from a by-value `for`.
    for d in [D(10), D(11), D(12), D(13)] {
        println!("for {}", d.0);
        if d.0 == 11 {
            break;
        }
    }
    println!("-- done3");
    // Early return while an iterator is alive.
    early();
    println!("-- early done");
    // A zip of unequal arrays pulls the mapped side only `min`
    // times; the excess drops with the zip.
    let z: Vec<(u8, u8)> = [D(20), D(21), D(22)]
        .into_iter()
        .map(|d| {
            println!("map {}", d.0);
            d.0
        })
        .zip([D(30)].into_iter().map(|d| d.0))
        .collect();
    println!("zip {:?}", z);
    // A partially consumed zip drops both sides' remainders.
    let mut z5 = [D(1), D(2), D(3)].into_iter().zip([D(4), D(5), D(6)].into_iter());
    println!("z5 first {:?}", z5.next().map(|(a, b)| (a.0, b.0)));
    drop(z5);
    println!("-- z5 dropped");
    // Chain and take adapters drop what they still own.
    let mut ch = [D(1), D(2)].into_iter().chain([D(3), D(4)].into_iter());
    println!("ch {:?}", ch.next().map(|d| d.0));
    drop(ch);
    println!("-- ch dropped");
    let mut t = [D(1), D(2), D(3)].into_iter().take(2);
    println!("t {:?}", t.next().map(|d| d.0));
    drop(t);
    println!("-- t dropped");
    // `nth` drops the skipped elements at the call.
    let mut it5 = [D(1), D(2), D(3), D(4), D(5)].into_iter();
    println!("nth1 {:?}", it5.nth(1).map(|d| d.0));
    println!("rem {:?}", it5.as_slice().iter().map(|d| d.0).collect::<Vec<_>>());
    drop(it5);
    println!("-- done nth");
    // A zero-sized element with `Drop` still drops once per element.
    let mut zit = [Z, Z, Z].into_iter();
    let _z = zit.next();
    drop(zit);
    println!("-- z done");
    // Nested arrays: the yielded row drops with its own owner.
    let nested = [[D(1), D(2)], [D(3), D(4)]];
    let mut nit = nested.into_iter();
    let first = nit.next().unwrap();
    println!("first {} {}", first[0].0, first[1].0);
    drop(nit);
    drop(first);
    println!("-- nested done");
}

struct Z;
impl Drop for Z {
    fn drop(&mut self) {
        println!("zdrop");
    }
}
