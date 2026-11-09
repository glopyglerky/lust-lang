// The C++ half of lust's panics under unwind tables (runtime/panic.c mode 2). A panic throws
// a private type and only catch points catch it, so the platform's C++ personality, tables
// and two-phase search do the unwinding. The generated units stay C, built with -fexceptions:
// their cleanup variables (cgen's frame structs and guards) run while it unwinds. Built by the
// release C++ compiler next to the C units; `lust build --emit-c <prefix>` writes it as
// <prefix>.eh.cc. The tcc dev build never uses it.
#include <cstdint>

namespace {
// The payload is the runtime's exception record (panic.c `struct lust_uexc`).
struct LustPanic {
    void *x;
};
}  // namespace

extern "C" [[noreturn]] void lust_unw_cxx_throw(void *x) { throw LustPanic{x}; }

// Runs f(ctx); 1 with the record in *caught when a panic unwound to here, else 0.
extern "C" int32_t lust_unw_cxx_try(void (*f)(void *), void *ctx, void **caught) {
    try {
        f(ctx);
        return 0;
    } catch (LustPanic &p) {
        *caught = p.x;
        return 1;
    }
}
