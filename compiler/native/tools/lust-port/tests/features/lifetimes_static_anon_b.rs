// feature: lifetimes.static_anon
// form: Formatter<'_> in a Display impl, the most common std carrier use
struct P { x: i32 }
impl std::fmt::Display for P {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "p{}", self.x) }
}
fn main() { println!("{}", P { x: 3 }); }
