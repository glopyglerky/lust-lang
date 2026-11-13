struct S;
fn setup() {
    impl S {
        pub fn g() -> u32 {
            2
        }
    }
}
fn main() {
    setup();
    println!("{}", S::g());
}
