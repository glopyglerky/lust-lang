// A user trait's method next to a std item of the same name: rustc's
// probe order decides (check/std_inherent.rs). Each step probes by
// value, then `&`, then `&mut`; in one probe std's inherent item wins.
trait ByValueLen {
    fn len(self) -> usize;
}
// The by-value probe of `Vec<u8>` finds this before Vec's `&self` len.
impl ByValueLen for Vec<u8> {
    fn len(self) -> usize {
        99
    }
}
trait First {
    fn first(&self) -> i32;
    fn get(&self, i: usize) -> i32;
}
// The `&Vec<u8>` probe finds these before the slice step's `first`/`get`.
impl First for Vec<u8> {
    fn first(&self) -> i32 {
        42
    }
    fn get(&self, i: usize) -> i32 {
        i as i32 + 100
    }
}
trait RefLen {
    fn len(&self) -> usize;
    fn floor(self) -> f64;
}
// Same probe as String's `&self` len and f64's by-value floor: std wins.
impl RefLen for String {
    fn len(&self) -> usize {
        7
    }
    fn floor(self) -> f64 {
        0.0
    }
}
impl RefLen for f64 {
    fn len(&self) -> usize {
        8
    }
    fn floor(self) -> f64 {
        -1.0
    }
}
trait Frob {
    fn frob(&self) -> u8;
}
// No std item of the name: the trait answers.
impl Frob for Vec<u8> {
    fn frob(&self) -> u8 {
        5
    }
}
fn main() {
    let v = vec![1u8, 2, 3];
    println!("{} {} {}", First::first(&v), v.first(), v.get(1));
    println!("{}", v.frob());
    let s = String::from("abc");
    println!("{} {}", s.len(), RefLen::len(&s));
    let x = 2.5f64;
    println!("{} {}", x.floor(), RefLen::floor(x));
    println!("{}", v.len());
}
