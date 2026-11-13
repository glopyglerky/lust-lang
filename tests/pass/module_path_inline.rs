// module_path! in the root and in an inline module of a single
// file: the crate segment is the crate name (the file stem here),
// not the stem of the file that holds the call (fz2-platform).
mod m {
    pub fn f() {
        println!("{}", module_path!());
    }
    pub mod inner {
        pub fn g() {
            println!("{}", module_path!());
        }
    }
}
fn main() {
    println!("{}", module_path!());
    m::f();
    m::inner::g();
}
