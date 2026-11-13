// `<u8 as Id>::item` inside a module names the module's trait `Id`,
// not the crate root's trait of the same name.
trait Id {
    fn id() -> u32;
    const C: u32;
}
impl Id for u8 {
    fn id() -> u32 {
        1
    }
    const C: u32 = 1;
}
mod m {
    pub trait Id {
        fn id() -> u32;
        const C: u32;
    }
    impl Id for u8 {
        fn id() -> u32 {
            2
        }
        const C: u32 = 2;
    }
    pub trait Only {
        fn only() -> u32;
    }
    impl Only for u16 {
        fn only() -> u32 {
            5
        }
    }
    pub fn f() -> u32 {
        <u8 as Id>::id() + <u16 as Only>::only()
    }
    pub fn c() -> u32 {
        <u8 as Id>::C
    }
    pub fn pat(x: u32) -> bool {
        matches!(x, <u8 as Id>::C)
    }
}
fn main() {
    println!("{} {} {} {}", m::f(), m::c(), m::pat(2), <u8 as Id>::id());
}
