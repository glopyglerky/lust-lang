use crate::prelude::*;


// PERF: a text precheck for check_unmatched_js_doc_parameters. It walks the
// same hosts as get_all_js_doc_tags. The JSDoc of a host is parsed from the
// comment ranges at host.pos(), and those ranges all end before
// skip_trivia(host.pos()). A parameter tag name is `param`, `arg` or
// `argument` right after `@`, or an identifier with a unicode escape. False
// means that no host can have a JsDocParameterTag. True is conservative.

// Go: checker/jsdoc.go:86 getAllJSDocTags
