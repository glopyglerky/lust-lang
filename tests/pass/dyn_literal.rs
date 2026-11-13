// ARCH 2.8: integer literals unsize like rustc: the one integer impl, else i32.
trait Show { fn show(&self) -> String; }
impl Show for u8 { fn show(&self) -> String { format!("u8:{}", self) } }
impl Show for String { fn show(&self) -> String { format!("s:{}", self) } }
trait Two { fn two(&self) -> String; }
impl Two for i32 { fn two(&self) -> String { format!("i32:{}", self) } }
impl Two for i64 { fn two(&self) -> String { format!("i64:{}", self) } }
fn main() {
    let a: &dyn Show = &3;
    let b: &dyn Two = &4;
    let c = String::from("x");
    let d: &dyn Show = &c;
    println!("{} {} {}", a.show(), b.two(), d.show());
}
