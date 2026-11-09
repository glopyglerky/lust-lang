mod t {
    pub trait Tr {
        fn go(&self) -> i32;
    }
    impl Tr for i32 {
        fn go(&self) -> i32 {
            *self + 1
        }
    }
}

fn mk() -> impl Copy {
    5i32
}
fn total() -> i32 {
    let _o = mk();
    let x: i32 = 4;
    x.go()
}
fn main() {
    println!("{}", total());
}
