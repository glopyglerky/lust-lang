trait Tr { fn v(&self) -> u32; fn name(&self) -> String { format!("tr{}", self.v()) } }
struct Out(u32);
fn show(t: &dyn Tr) -> String { t.name() }
fn main() {
    println!("{}", show(&Out(5)));
    println!("{}", Out(2).extra());
    setup();
}
fn setup() {
    impl Tr for Out { fn v(&self) -> u32 { 1 } }
    impl Out { fn extra(&self) -> u32 { self.0 * 3 } }
}
impl Tr for Out { fn v(&self) -> u32 { self.0 + 300 } }
