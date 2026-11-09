// A lang trait impl (`PartialEq`) on a struct with a lifetime param.
#[derive(Debug)]
struct Item<'a> { name: &'a str }
struct Bag<'a> { items: Vec<Item<'a>> }
impl<'a> Bag<'a> {
    fn first(&self) -> Option<&Item<'a>> { self.items.first() }
}
impl<'a> PartialEq for Bag<'a> { fn eq(&self, o: &Self) -> bool { self.items.len() == o.items.len() } }
fn main() {
    let s = String::from("x");
    let b = Bag { items: vec![Item { name: &s }] };
    let c = Bag { items: vec![] };
    println!("{:?} {}", b.first(), b.eq(&c));
}
