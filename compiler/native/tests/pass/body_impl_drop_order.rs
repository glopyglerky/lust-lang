// Drop impls on a block type with a lifetime: drop order matches rustc.
struct Outer(&'static str);
impl Drop for Outer { fn drop(&mut self) { println!("outer {}", self.0); } }
fn main() {
    let _a = Outer("a");
    {
        struct G<'a>(&'a mut Vec<u8>, u8);
        impl Drop for G<'_> { fn drop(&mut self) { self.0.push(self.1); println!("g {}", self.1); } }
        let mut v = Vec::new();
        {
            let _x = G(&mut v, 1);
        }
        let _y = G(&mut v, 2);
        let _z = Outer("z");
    }
    let _b = Outer("b");
}
