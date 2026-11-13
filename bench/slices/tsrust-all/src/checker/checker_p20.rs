//! Port of `checker/checker.go` lines 17614-18535: binding element and
//! binding pattern types, assignment declaration types, implicit any
//! reporting, type widening, and accessor and alias symbol types.

use crate::prelude::*;


impl WideningContext {
    // Go: checker/checker.go:18790 WideningContext.getChildContext
    // PORT: the child keeps a `Weak` to its parent, so this takes the
    // parent's `Rc` instead of `&self`. Call as
    // `WideningContext::get_child_context(&ctx, name)`.
    pub fn get_child_context(
        this: &Rc<RefCell<WideningContext>>,
        property_name: &str,
    ) -> Rc<RefCell<WideningContext>> {
        if let Some(cached) = this.borrow().child_contexts.get(property_name) {
            return cached.clone();
        }
        let result = Rc::new(RefCell::new(WideningContext {
            parent: Some(Rc::downgrade(this)),
            property_name: property_name.to_string(),
            ..Default::default()
        }));
        this.borrow_mut()
            .child_contexts
            .insert(property_name.to_string(), result.clone());
        result
    }
}

