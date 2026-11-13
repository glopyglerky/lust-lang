// Type aliases and consts are not in the trait-system key: what reads
// them records a read, and impl headers, bounds and trait records fold in
// resolved. Each version edits one alias or const; the bodies stay.
type Id = u64;
type Name = &'static str;
type Unused = Vec<u8>;
const N: usize = 4;
const SCALE: i64 = 100;

trait Tag {
    fn tag(&self) -> Name;
}

impl Tag for Id {
    fn tag(&self) -> Name {
        "id"
    }
}

impl Tag for [u8; N] {
    fn tag(&self) -> Name {
        "triple"
    }
}

trait Measure {
    fn measure(&self) -> i64;
}

impl<T: Tag> Measure for Vec<T> {
    fn measure(&self) -> i64 {
        self.len() as i64 * SCALE
    }
}

fn first<const K: usize>(a: [i64; K]) -> i64 {
    a[0] * K as i64
}

fn show_id(x: u32) -> Name {
    x.tag()
}

fn show_arr() -> Name {
    [0u8; 4].tag()
}

fn total() -> i64 {
    vec![1u32, 2].measure() + first::<N>([2; N])
}

fn main() {
    println!("{} {} {}", show_id(7), show_arr(), total());
}
