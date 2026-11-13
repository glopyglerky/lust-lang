// K6: libbase-style unit. Compiled -shared by tcc into libbase.so. It calls
// the Rust shim, so the link shape matches the daemon (base + overlay exe).
#include "shim.h"
long base_compute(long x) {
    char msg[64];
    int caught = shim_catch_panic(msg, sizeof msg);
    long sum = 0;
    int rc = shim_spawn_sum(4, &sum);
    return shim_add(x, caught * 1000 + (rc == 0 ? sum : -1));
}
