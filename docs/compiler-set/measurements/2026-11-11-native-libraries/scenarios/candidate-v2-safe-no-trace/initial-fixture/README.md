# Supplemental safe primitive library diagnostics

This is a distinct, fixed source fixture for the bounded safe primitive library
implementation. Run `tools/diagnose-native-library-safe.py`. It exercises actual
Cargo separate compilation, source-hidden artifact imports, public signatures,
scalar ABI boundaries, invalidation, corrupt archives and explicit unsupported
public contracts. It does not transform the required fixture at run time.

The separate `tests/native-library` fixture retains its required unsafe producer
and consumer gates. Success here does not replace those gates, establish full
native Rust, or produce `qualified=true`. Local runs without tracing are also
marked `no_trace_qualification=true`. No timings are performance evidence.
