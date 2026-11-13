// expect: L0106
struct G;
impl G { const C: u8 = 1; const C: u8 = 2; }
fn main(){ println!("{}", G::C); }
