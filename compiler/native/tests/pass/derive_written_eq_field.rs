// Derived PartialEq, tuple, Vec, and Option equality over a field type with
// a written PartialEq (no ne of its own): the glue calls its eq, in
// rustc's order; != is core's default (!eq).
#[derive(Debug)]
struct Name(String);
impl PartialEq for Name { fn eq(&self, o: &Name) -> bool { println!("eq {} {}", self.0, o.0); self.0.to_lowercase() == o.0.to_lowercase() } }
#[derive(Debug, PartialEq)]
struct Rec { n: Name, id: u8 }
#[derive(Debug, PartialEq)]
enum E { A(Name), B }
fn main() {
    let a = Rec { n: Name(String::from("Ab")), id: 1 };
    let b = Rec { n: Name(String::from("aB")), id: 1 };
    println!("{} {}", a == b, a != b);
    println!("{}", E::A(Name(String::from("x"))) == E::A(Name(String::from("X"))));
    println!("{}", vec![Name(String::from("q"))] == vec![Name(String::from("Q"))]);
    println!("{}", (Name(String::from("z")), 1) != (Name(String::from("Z")), 1));
    println!("{}", Some(Name(String::from("m"))) == None);
    println!("{}", Name(String::from("k")) == Name(String::from("K")));
}
