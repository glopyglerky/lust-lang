use crate::prelude::*;


// Go: checker/checker.go:27109 compareTypeIds
pub fn compare_type_ids(t1: TypeId, t2: TypeId) -> i32 {
    t1.index() as i32 - t2.index() as i32
}


// Go: checker/checker.go:27261 isInvalidComputedPropertyName

