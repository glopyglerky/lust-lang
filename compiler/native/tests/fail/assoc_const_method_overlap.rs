// expect: L0106
struct G<T>(T);
impl G<i32> { fn C() -> u8 { 9 } }
impl G<i32> { const C: u8 = 1; }
fn main(){ println!("{}", G::<i32>::C); }
