//! Port of Go `ls/host.go`.

use crate::ls::prelude::*;

// Go: ls/host.go:10 Host
// PORT: Go `*lsconv.Converters` is shared (`Rc`). Go `*sourcemap.ECMALineInfo`
// and `*autoimport.Registry` can be nil, so they are `Option<Rc<..>>`.
// `[]string` params are `&[String]`; a Go `nil` slice is `&[]`. The
// `read_file` text is a `FileText`, shared like a Go string: a language
// service reads a file once per location it converts (`get_script`), and a
// copy of a big file per location made references 7 times slower than Go.
