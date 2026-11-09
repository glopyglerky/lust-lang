// A fn-body glob of Ordering's variants.
use std::cmp::Ordering;
mod inner {
    pub fn f() -> bool {
        use std::cmp::Ordering::*;
        matches!(1.cmp(&2), Less)
    }
}
fn main() {
    println!("{} {:?}", inner::f(), Ordering::Equal);
}
