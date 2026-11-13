// A module's #![recursion_limit] is ignored with a warning in rustc, even a low one,
// so it neither lowers the limit nor makes lust reject the crate.
mod m {
    #![recursion_limit = "4"]
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
