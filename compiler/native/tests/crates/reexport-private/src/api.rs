pub use crate::hidden::Secret;

pub fn make() -> Secret {
    crate::hidden::build()
}
