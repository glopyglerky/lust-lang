use crate::prelude::*;
use crate::smallvec::SmallVec;

// PORT: cross-file decisions for this range.
// - Go `*TypeAlias` parameters and returns that store or create an alias are
//   `Option<Rc<TypeAlias>>` (the `Type::alias` field type).
// - The cache key functions `getTypeInstantiationKey` and
//   `getConditionalTypeKey` only read the alias, so they take
//   `Option<&TypeAlias>` (pass `alias.as_deref()`). They are free functions
//   because they only hash ids.
// - Go `*ConditionalRoot` is `Rc<RefCell<ConditionalRoot>>` (see types.rs), so
//   `get_conditional_type` takes it by value (a cloned `Rc`).


// Go: checker/checker.go:23080 getModifiedReadonlyState
pub fn get_modified_readonly_state(state: bool, modifiers: MappedTypeModifiers) -> bool {
    if modifiers.intersects(MappedTypeModifiers::INCLUDE_READONLY) {
        return true;
    } else if modifiers.intersects(MappedTypeModifiers::EXCLUDE_READONLY) {
        return false;
    }
    state
}


// Go: checker/checker.go:23472 isSimpleIdentifierTypeReference
