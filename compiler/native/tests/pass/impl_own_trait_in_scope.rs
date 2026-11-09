// Inside a trait impl, that trait is in scope for every receiver (rustc's current
// trait ref): its methods, their closures and nested fns, with no `use` of the trait.
mod t {
    pub trait Tr {
        fn a(&self) -> u32;
        fn b(&self) -> u32 {
            1
        }
    }
    impl Tr for u32 {
        fn a(&self) -> u32 {
            *self
        }
    }
}

mod m {
    pub struct S(pub u32);
    impl crate::t::Tr for S {
        fn a(&self) -> u32 {
            let f = || 5u32.b();
            fn inner() -> u32 {
                7u32.a()
            }
            self.b() + 4u32.a() + f() + inner()
        }
    }

    pub struct W(pub Vec<u8>);
    impl std::io::Write for W {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.write_all(b"!")
        }
    }
}

fn main() {
    use std::io::Write;
    use t::Tr;
    println!("{}", m::S(0).a());
    let mut w = m::W(Vec::new());
    w.flush().unwrap();
    println!("{:?}", w.0);
}
