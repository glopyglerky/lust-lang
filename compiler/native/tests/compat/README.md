# K1 compatibility-spike fixtures

Standalone programs, one per K1 shape (docs/ARCH.md section 10,
DESIGN-TS spike list). Each is reduced from the real ts-rust code
named in its header and keeps the type and control shapes that make
the shape hard. Expected output is from rustc (edition 2024); the
panic mode each fixture needs is the `panic-mode:` line in its
header. scripts/test-kernels.sh runs them in both modes (rustc
validates, `LUST=path` reports unsupported without failing today).

| Fixture | Shape | ts-rust source |
|---|---|---|
| visitor_callbacks.rs | NodeVisitor with stored Rc<dyn Fn> callbacks and a borrowed visitor (the K8 hard half) | ast/visitor.rs |
| arenabox_deref.rs | ArenaBox with Deref, DerefMut, Clone, Debug; arena values never drop | checker/types.rs |
| blanket_anyvalue.rs | AnyValue blanket impl with dyn Any downcast, incl. the double-box trap | frontend/json_ext.rs |
| go_flags_types.rs | go_flags! and go_enum! output types (macros verbatim) | flags_macros.rs, flags.rs |
| scoped_thread.rs | thread::scope workers over a borrowed slice | program.rs bind loop |
| recovered_panic.rs | catch_unwind with drops, nested catch, lock poison (panic=unwind) | program.rs job runners |
| fileref_guard.rs | FileRef guard-backed borrow, unsized targets | ast/file_version.rs |
| rc_self_method.rs | self: &Rc<Self> receiver | frontend/packagejson.rs |

Not in this set: the K1 C shim item and the borrowed async task
item. ARCH 9.3 schedules those as bounded probes at wave C and D
entries (section 10 also splits the async item out as K9). They get
fixtures with those waves, not here.
