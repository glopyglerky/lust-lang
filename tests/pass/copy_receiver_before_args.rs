// A by-value `Copy` receiver is read before the arguments run (R12).
#[derive(Clone, Copy)]
struct Tr(u32);
struct C { tracer: Option<Tr>, n: u32 }
impl C {
    fn bump(&mut self) -> u32 { self.n += 1; self.n }
    fn go(&mut self) -> Option<u32> { self.tracer.map(|tr| { let k = self.bump(); tr.0 + k }) }
}
fn main() { let mut c = C { tracer: Some(Tr(5)), n: 0 }; println!("{:?} {}", c.go(), c.n); }
