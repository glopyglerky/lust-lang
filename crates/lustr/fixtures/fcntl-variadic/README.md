# Variadic `fcntl` and Rust-owned file descriptors

This Unix fixture uses libc's real `fcntl(int, int, ...)` declaration with both
`F_GETFD` (two arguments) and `F_SETFD` (three arguments), checks that descriptor
flags can be toggled and restored, clones the `std::fs::File`, reads from both
handles and drops them through std's ownership implementation. It has no
registry dependencies.

Native Lust previously declared `fcntl` with three fixed integer parameters in
its client, daemon and image code. That disagreed with libc and std's two-fixed-
parameter variadic declaration. The pinned Cranelift backend rejects the two
incompatible imports when they occur in the same codegen unit. A small program
using the fixed declaration plus `File::drop` reproduced the failure with the
default codegen settings; changing only the declaration to variadic fixed it.

The integration tests compare ordinary Cargo and coordinated execution. The
optional `actual_cranelift_fcntl_variadic_calls_share_std_file_descriptor_abi`
gate verifies fresh compiler invocations before running the executable and test
harness with the pinned backend. It does not disable std's descriptor checks,
change codegen-unit counts, substitute syscalls or retry a failed build through
another backend.

The [GNU C Library declaration](https://sourceware.org/glibc/manual/latest/html_node/Control-Operations.html)
uses two fixed arguments followed by an ellipsis. Native callers still pass the
same `F_SETFD` integer values after correcting their declarations; benchmarks
must identify the corrected source revision rather than treating it as the
previous unchanged source snapshot.
