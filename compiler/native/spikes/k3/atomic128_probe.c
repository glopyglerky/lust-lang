// K3: 16-byte atomics. tcc must reject this at compile time (size > 8 in
// tccgen). cc needs -latomic (lock-based) or -mcx16. Prints lock-free status.
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
struct pair16 { uint64_t lo, hi; };
int main(void) {
    _Atomic struct pair16 v;
    struct pair16 z = {0, 0}, n = {11, 22}, e = z;
    atomic_init(&v, z);
    int lf = atomic_is_lock_free(&v);
    int ok = atomic_compare_exchange_strong(&v, &e, n);
    struct pair16 got = atomic_load(&v);
    printf("INFO atomic128 lock_free=%d cas_ok=%d got=%llu,%llu\n", lf, ok,
           (unsigned long long)got.lo, (unsigned long long)got.hi);
    int pass = ok && got.lo == 11 && got.hi == 22;
    printf("%s atomic128_cas\nRESULT %s fails=%d\n", pass ? "PASS" : "FAIL", pass ? "PASS" : "FAIL", pass ? 0 : 1);
    return pass ? 0 : 1;
}
