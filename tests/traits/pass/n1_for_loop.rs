// ARCH 2.4 step 3, the N1 shape: one concrete IntoIterator impl
// and the bounded bare-param std blanket both match by header.
// Local does not implement Iterator, so the blanket clause is
// provably false and the concrete impl must be selected. The
// selection happens through a `for` loop. From the round-2
// probe select.rs, with the local-trait half kept below.
struct LocalIt {
    left: u8,
}
impl Iterator for LocalIt {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.left == 0 {
            None
        } else {
            self.left -= 1;
            Some(7)
        }
    }
}
struct Local;
impl IntoIterator for Local {
    type Item = u8;
    type IntoIter = LocalIt;
    fn into_iter(self) -> LocalIt {
        LocalIt { left: 2 }
    }
}

// The same shape with local traits: no `for` syntax, so the
// selection goes through a generic function bound.
trait Iter {
    fn next(&mut self) -> Option<u8>;
}
trait IntoIter {
    type It: Iter;
    fn into(self) -> Self::It;
}
impl<I: Iter> IntoIter for I {
    type It = I;
    fn into(self) -> I {
        self
    }
}
struct Local2;
struct Local2It;
impl Iter for Local2It {
    fn next(&mut self) -> Option<u8> {
        Some(9)
    }
}
impl IntoIter for Local2 {
    type It = Local2It;
    fn into(self) -> Local2It {
        Local2It
    }
}
fn get<I: IntoIter>(v: I) -> u8 {
    v.into().next().unwrap()
}
fn main() {
    let mut sum = 0u8;
    for x in Local {
        sum += x;
    }
    println!("{sum} {}", get(Local2));
}
