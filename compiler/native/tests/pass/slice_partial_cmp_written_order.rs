// `partial_cmp` on slices, arrays, Vecs, and tuples whose element has a written PartialOrd
// beside an Ord impl: the element's partial_cmp decides (here it reverses Ord), and a None
// from it makes the whole comparison None. ts-rust: SmallVec<[(Name, SymbolId); 16]>.
use std::cmp::Ordering;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Name(u32);
impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}
impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
struct Maybe(i32);
impl PartialOrd for Maybe {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == 0 || other.0 == 0 { None } else { self.0.partial_cmp(&other.0) }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Sym(u32);

fn main() {
    let a = [Name(1), Name(5)];
    let b = [Name(2), Name(0)];
    println!("{:?} {:?}", a[..].partial_cmp(&b[..]), a.partial_cmp(&b));
    let v: Vec<(Name, Sym)> = vec![(Name(1), Sym(2))];
    let w: Vec<(Name, Sym)> = vec![(Name(1), Sym(3))];
    println!("{:?} {:?}", v.partial_cmp(&w), v.as_slice().partial_cmp(w.as_slice()));
    println!("{:?}", (Name(1), Sym(1)).partial_cmp(&(Name(2), Sym(1))));
    let m = vec![Maybe(1), Maybe(0)];
    let n = vec![Maybe(1), Maybe(4)];
    println!("{:?} {:?}", m.partial_cmp(&n), m[..1].partial_cmp(&n[..1]));
    let bx: Box<[Name]> = vec![Name(3)].into();
    let by: Box<[Name]> = vec![Name(3), Name(1)].into();
    println!("{:?}", bx.partial_cmp(&by));
}
