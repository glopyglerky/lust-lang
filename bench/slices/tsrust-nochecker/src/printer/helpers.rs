//! Port of `printer/helpers.go`.

use crate::prelude::*;

// Go: printer/helpers.go:3 Priority
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Priority {
    pub value: i32,
}

// Go: printer/helpers.go:7 EmitHelper
// PORT: Go `*EmitHelper` values are package-level pointers compared by
// identity. Here they are statics used as `&'static EmitHelper` and compared
// with `std::ptr::eq`. `TextCallback` is a plain fn pointer that takes the
// Go `makeUniqueName` callback. `Priority` is `Option<Priority>` (Go nil is None).

// Go: printer/helpers.go:17 compareEmitHelpers
// TypeScript Helpers
// Go: printer/helpers.go:35 decorateHelper

// Go: printer/helpers.go:48 metadataHelper

// Go: printer/helpers.go:58 paramHelper

// ESNext Helpers
// Go: printer/helpers.go:70 addDisposableResourceHelper

// Go: printer/helpers.go:98 disposeResourcesHelper

// Class Fields Helpers
// Go: printer/helpers.go:184 classPrivateFieldGetHelper
// See the Go doc comment (helpers.go:136) for the parameters and uses.

// Go: printer/helpers.go:246 classPrivateFieldSetHelper
// See the Go doc comment (helpers.go:195) for the parameters and uses.

// Go: printer/helpers.go:270 classPrivateFieldInHelper
// See the Go doc comment (helpers.go:258) for the parameters and uses.

// ES2018 Helpers
// Go: printer/helpers.go:282 awaitHelper

// Go: printer/helpers.go:289 asyncGeneratorHelper

// Go: printer/helpers.go:308 asyncDelegatorHelper

// Go: printer/helpers.go:320 asyncValuesHelper

// ES2018 Destructuring Helpers
// Go: printer/helpers.go:334 restHelper

// Go: printer/helpers.go:351 awaiterHelper

// Go: printer/helpers.go:367 AsyncSuperHelper

// Go: printer/helpers.go:370 AsyncSuperHelper.TextCallback
fn async_super_helper_text_callback(make_unique_name: &mut dyn FnMut(&str) -> String) -> String {
    "\nconst ".to_string() + &make_unique_name("_superIndex") + " = name => super[name];"
}

// Go: printer/helpers.go:375 AdvancedAsyncSuperHelper

// Go: printer/helpers.go:378 AdvancedAsyncSuperHelper.TextCallback
fn advanced_async_super_helper_text_callback(
    make_unique_name: &mut dyn FnMut(&str) -> String,
) -> String {
    "\nconst ".to_string()
        + &make_unique_name("_superIndex")
        + " = (function (geti, seti) {\n"
        + "    const cache = Object.create(null);\n"
        + "    return name => cache[name] || (cache[name] = { get value() { return geti(name); }, set value(v) { seti(name, v); } });\n"
        + "})(name => super[name], (name, value) => super[name] = value);"
}

// ES Decorator Helpers
// Go: printer/helpers.go:388 esDecorateHelper

// Go: printer/helpers.go:422 runInitializersHelper

// ES2015 Helpers
// Go: printer/helpers.go:438 makeTemplateObjectHelper

// Go: printer/helpers.go:449 propKeyHelper

// https://tc39.es/ecma262/#sec-setfunctionname
// Go: printer/helpers.go:459 setFunctionNameHelper

// ES Module Helpers
// Go: printer/helpers.go:471 createBindingHelper

// Go: printer/helpers.go:489 setModuleDefaultHelper

// Go: printer/helpers.go:501 importStarHelper

// Go: printer/helpers.go:526 importDefaultHelper

// Go: printer/helpers.go:535 exportStarHelper

// Go: printer/helpers.go:546 rewriteRelativeImportExtensionsHelper
