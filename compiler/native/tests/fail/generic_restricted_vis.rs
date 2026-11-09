// expect: L0510
mod a {
    pub mod inner {
        pub struct Deep {
            pub(super) up: i32,
        }
        impl Deep {
            pub fn new() -> Deep {
                Deep { up: 1 }
            }
        }
    }
}
fn get<T>(t: T) -> i32 {
    let _ = &t;
    a::inner::Deep::new().up
}
fn main() {
    println!("{}", get(3));
}
