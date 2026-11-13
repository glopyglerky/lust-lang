struct S;
fn setup() {
    impl S {
        pub fn g() -> u32 {
            1
        }
    }
}
fn main() {
    setup();
    println!("{}", S::g());
}
