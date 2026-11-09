mod util { pub fn k() -> u32 { 1 } }
trait Named { fn name(&self) -> u32; }
fn show<T: Named>(t: &T) -> u32 { t.name() + 1 }
fn site(n: u32) -> u32 {
    struct Guard(u32);
    impl Drop for Guard {
        fn drop(&mut self) {
            println!("drop {}", self.0 + 1);
        }
    }
    struct Pt { x: u32 }
    impl Pt {
        fn get(&self) -> u32 {
            self.x * 2
        }
    }
    impl Named for Pt {
        fn name(&self) -> u32 { self.x + 5 }
    }
    let extra = n + 1;
    let _g = Guard(extra);
    let p = Pt { x: n };
    p.get() + show(&p) + util::k()
}
fn main() { println!("{}", site(3)); println!("{}", site(4)); }
