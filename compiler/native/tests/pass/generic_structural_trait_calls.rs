// A generic `T: Clone` / `Ord` / `PartialOrd` / `Default` call whose
// instance has a tuple or array Self: selection answers structurally, and
// the instance takes the builtin body (it used to instantiate the trait
// method stub: an internal error).

fn d<T: Default>() -> T { T::default() }
fn c2<T: Clone>(x: &T) -> T { x.clone() }
fn cmpp<T: PartialOrd>(a: &T, b: &T) -> Option<std::cmp::Ordering> { a.partial_cmp(b) }
fn mx<T: Ord>(a: T, b: T) -> T { a.max(b) }
fn cl<F: Fn() -> u8 + Clone>(f: &F) -> u8 { let g = f.clone(); g() }
fn main() {
    let a: (u8, bool) = d();
    let b: [i32; 3] = d();
    println!("{:?} {:?} {:?}", a, b, c2(&(String::from("q"), 2u8)));
    println!("{:?} {:?}", cmpp(&(1.5f64, 2u8), &(1.5f64, 1u8)), mx((1u8, 'z'), (2u8, 'a')));
    let k = 5u8;
    println!("{}", cl(&move || k + 1));
}
