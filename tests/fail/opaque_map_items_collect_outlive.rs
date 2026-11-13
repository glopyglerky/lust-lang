// expect: L0306
// rustc E0597: the collected items of a returned `map` borrow `d`.
pub struct D {
    v: Vec<String>,
}
impl D {
    fn it(&self) -> impl Iterator<Item = &String> + '_ {
        self.v.iter().map(|x| x)
    }
}
fn main() {
    let c: Vec<_>;
    {
        let d = D { v: vec![String::from("a")] };
        c = d.it().collect();
    }
    println!("{:?}", c);
}
