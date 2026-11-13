// A body impl's methods see the block's consts, nested fns, and their own assoc consts.
fn main() {
    const BASE: u32 = 10;
    fn helper(x: u32) -> u32 { x + 1 }
    struct Acc { v: u32 }
    impl Acc {
        const STEP: u32 = 3;
        fn bump(&mut self) { self.v = helper(self.v) + BASE + Self::STEP; }
    }
    let mut a = Acc { v: 0 };
    a.bump(); a.bump();
    println!("{}", a.v);
}
