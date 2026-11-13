// rustc E0433: the local module `util` shadows the dependency `util`,
// and it has no macro `m`.
mod util {
    pub fn g() {}
}

fn main() {
    util::g();
    println!("{}", util::m!());
}
