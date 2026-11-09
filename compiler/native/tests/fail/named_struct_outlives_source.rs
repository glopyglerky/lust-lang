// expect: L0306
// rustc E0597: the struct holds a borrow of `n`, which dies first.
struct Finder<'n> {
    needle: &'n [u8],
}

impl<'n> Finder<'n> {
    fn new(needle: &'n [u8]) -> Finder<'n> {
        Finder { needle }
    }
}

fn main() {
    let f;
    {
        let n = vec![1u8, 2];
        f = Finder::new(&n);
    }
    println!("{:?}", f.needle);
}
