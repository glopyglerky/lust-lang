// Drop points of owned items under the adapters: rejected items drop
// at once, `vec::IntoIter` drops what it still owns, `Chain` drops a
// spent side when it moves on, `max` drops the losers.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn ds(ns: &[u32]) -> Vec<D> {
    ns.iter().map(|n| D(*n)).collect()
}
fn id(d: &Option<D>) -> u32 {
    match d {
        Some(d) => d.0,
        None => 0,
    }
}
fn main() {
    let kept: Vec<D> = ds(&[1, 2, 3, 4]).into_iter().filter(|d| d.0 % 2 == 0).collect();
    println!("kept {}", kept.len());
    let mut it = ds(&[10, 11, 12]).into_iter();
    let first = it.next();
    println!("took {}", id(&first));
    drop(it);
    println!("iterator gone");
    let tail = ds(&[20, 21, 22]).into_iter().rev().nth(1);
    println!("nth back {}", id(&tail));
    let marker = D(99);
    let c = ds(&[30]).into_iter().map(move |d| D(d.0 + marker.0)).chain(ds(&[40])).count();
    println!("chain count {}", c);
    let big = ds(&[5, 9, 2, 9, 1]).into_iter().max_by_key(|d| d.0);
    println!("max {}", id(&big));
    let small = ds(&[5, 2, 7, 2]).into_iter().min_by_key(|d| d.0);
    println!("min {}", id(&small));
    let some: Vec<D> = ds(&[50, 51, 52, 53]).into_iter().skip(1).take(2).collect();
    println!("skip take {}", some.len());
    let z: Vec<(D, D)> = ds(&[60, 61, 62]).into_iter().zip(ds(&[70, 71])).collect();
    println!("zip {}", z.len());
    println!("end");
}
