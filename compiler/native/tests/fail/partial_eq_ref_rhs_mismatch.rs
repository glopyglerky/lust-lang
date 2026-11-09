// expect: L0113
// &Meters has no PartialEq<u32>: the impl is on Meters, and == does
// not autoderef the left side through a reference to find it.
struct Meters(u32);

impl PartialEq<u32> for Meters {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

fn main() {
    let m = Meters(7);
    println!("{}", &m == 7u32);
}
