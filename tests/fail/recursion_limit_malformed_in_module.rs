// expect: L0001
// rustc: `limit` must be a non-negative integer. A module's limit is ignored, but a malformed
// one is still an error.
mod m {
    #![recursion_limit = "abc"]
    pub fn f() -> i32 {
        1
    }
}

macro_rules! one {
    () => {
        m::f()
    };
}

fn main() {
    println!("{}", one!());
}
