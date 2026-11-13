// ARCH 2.8: supertrait calls on `&dyn Sub`, default methods, and upcasts to each supertrait.
trait Named { fn name(&self) -> String; }
trait Animal: Named { fn legs(&self) -> u32; fn describe(&self) -> String { format!("{} has {} legs", self.name(), self.legs()) } }
trait Pet: Animal { fn owner(&self) -> String; }
struct Dog { n: u32 }
impl Named for Dog { fn name(&self) -> String { format!("dog{}", self.n) } }
impl Animal for Dog { fn legs(&self) -> u32 { 4 } }
impl Pet for Dog { fn owner(&self) -> String { String::from("ann") } }
struct Bird;
impl Named for Bird { fn name(&self) -> String { String::from("bird") } }
impl Animal for Bird { fn legs(&self) -> u32 { 2 } fn describe(&self) -> String { String::from("a bird") } }
impl Pet for Bird { fn owner(&self) -> String { String::from("bo") } }
fn show_named(n: &dyn Named) { println!("named {}", n.name()); }
fn show_animal(a: &dyn Animal) { println!("{} / {}", a.describe(), a.name()); show_named(a); }
fn main() {
    let d = Dog { n: 7 };
    let b = Bird;
    let pets: [&dyn Pet; 2] = [&d, &b];
    for p in &pets {
        println!("{} {} {}", p.name(), p.legs(), p.owner());
        let a: &dyn Animal = *p;
        show_animal(a);
        let n: &dyn Named = *p;
        show_named(n);
        println!("{}", p.describe());
    }
}
