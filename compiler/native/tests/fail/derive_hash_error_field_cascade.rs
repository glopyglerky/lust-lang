// expect: L0102
// rustc E0412: the unknown field type is the only error. A derive on
// a struct that holds this one by value proves its own field bounds
// through the broken field without a second, wrong derive error.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaId(NoSuchType);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeRef {
    pub arena: ArenaId,
    pub file: u32,
}
fn main() {}
