// expect: L0101
// rustc E0423: a type param is not a value. `T` here is neither the param
// nor `a::T`, which this module does not import.
mod a {
    pub struct T;
}

mod m {
    pub fn f<T>(t: T) -> T {
        let _x = T;
        t
    }
}

fn main() {
    let _ = a::T;
    println!("{}", m::f(1u8));
}
