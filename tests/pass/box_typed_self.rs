// A `self: Box<Self>` receiver matches by its written type.
struct P { x: i32 }
impl P {
    fn into_x(self: Box<Self>) -> i32 { self.x }
    fn by_val(self) -> i32 { self.x * 2 }
}
fn main() {
    let b = Box::new(P { x: 21 });
    println!("{}", b.into_x());
    println!("{}", P { x: 3 }.by_val());
}
