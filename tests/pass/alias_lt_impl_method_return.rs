// An impl method returns `R<'a>`: the result outlives the struct.
type R<'a> = &'a String;
struct H<'a> { r: R<'a> }
impl<'a> H<'a> { fn get(&self) -> R<'a> { self.r } }
fn main() {
    let s = String::from("h");
    let g;
    { let h = H { r: &s }; g = h.get(); }
    println!("{}", g);
}
