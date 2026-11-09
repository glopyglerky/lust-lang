// Impl and trait both return `&'static str`: the value can die first.
trait Tr { fn get(&self) -> &'static str; }
struct X { s: String }
impl Tr for X { fn get(&self) -> &'static str { "k" } }
fn main() {
    let r;
    { let x = X { s: String::from("a") }; r = x.get(); let _ = &x.s; }
    println!("{r}");
}
