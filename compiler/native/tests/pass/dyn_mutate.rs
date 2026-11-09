// ARCH 2.8: `&mut dyn` calls mutate the concrete value; reborrows keep the source usable.
trait Counter { fn bump(&mut self, by: u32); fn get(&self) -> u32; }
struct C { n: u32 }
impl Counter for C { fn bump(&mut self, by: u32) { self.n += by; } fn get(&self) -> u32 { self.n } }
struct D { v: Vec<u32> }
impl Counter for D { fn bump(&mut self, by: u32) { self.v.push(by); } fn get(&self) -> u32 { self.v.len() as u32 } }
fn bump_all(cs: &mut [&mut dyn Counter; 2]) { let mut i = 0; while i < 2 { cs[i].bump(3); i += 1; } }
fn twice(c: &mut dyn Counter) { c.bump(1); c.bump(1); }
fn main() {
    let mut c = C { n: 0 };
    let mut d = D { v: Vec::new() };
    {
        let r: &mut dyn Counter = &mut c;
        r.bump(5);
        twice(r);
        twice(r);
        println!("{}", r.get());
    }
    twice(&mut d);
    let shared: &dyn Counter = &mut d;
    println!("{} {}", shared.get(), c.n);
    let mut arr: [&mut dyn Counter; 2] = [&mut c, &mut d];
    bump_all(&mut arr);
    println!("{} {}", arr[0].get(), arr[1].get());
}
