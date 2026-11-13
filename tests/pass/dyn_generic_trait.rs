// ARCH 2.8: `dyn Tr<u32>` and `dyn Tr<u8>` on one type pick different impls.
trait Conv<T> { fn conv(&self, x: T) -> T; fn id(&self) -> u8 { 1 } }
struct Add(u32);
impl Conv<u32> for Add { fn conv(&self, x: u32) -> u32 { x + self.0 } }
impl Conv<u8> for Add { fn conv(&self, x: u8) -> u8 { x * 2 } fn id(&self) -> u8 { 2 } }
fn run(c: &dyn Conv<u32>, v: u32) -> u32 { c.conv(v) }
fn main() {
    let a = Add(10);
    println!("{}", run(&a, 5));
    let b: &dyn Conv<u8> = &a;
    println!("{} {}", b.conv(21), b.id());
    let c: &dyn Conv<u32> = &a;
    println!("{}", c.id());
}
