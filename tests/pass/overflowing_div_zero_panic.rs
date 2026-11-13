// Signed overflowing division by zero panics inside core, so the
// location is core's num/mod.rs, not this call.
#[inline(never)]
fn id<T>(x: T) -> T {
    x
}

fn main() {
    let r = id(5i32).overflowing_div(0).0;
    println!("{r}");
}
