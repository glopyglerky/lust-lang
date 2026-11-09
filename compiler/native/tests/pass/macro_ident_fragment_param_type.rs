// A macro-made method whose parameter type is an `ident` fragment
// (`n1: $ty`): the parameter stays visible to the macro's own body.
macro_rules! def {
    ($name:ident, $ty:ident) => {
        pub struct $name;
        impl $name {
            pub fn add(n1: $ty, n2: $ty) -> Option<$ty> {
                n1.checked_add(n2)
            }
            pub fn first(n1: $ty, n2: i8) -> $ty {
                let _ = n2;
                n1
            }
        }
    };
}
def!(Hour, i8);
def!(Big, i64);

fn main() {
    println!("{:?} {:?} {:?} {}", Hour::add(100, 100), Hour::add(1, 2), Big::add(1, 2), Big::first(7, 0));
}
