// K3: C11 fetch ops directly on _Atomic(T *). tcc rejects these at compile
// time ("integral or integer-sized pointer target type expected", from the
// size/btype check in tccgen.c). Under cc they compile, and the arithmetic
// is in bytes, not elements. Lust codegen rule: do pointer RMW on uintptr_t.
#include <stdatomic.h>
#include <stdio.h>
static int cells[4];
int main(void) {
    _Atomic(int *) p;
    atomic_init(&p, &cells[0]);
    int *old = atomic_fetch_add(&p, 2);
    int *now = atomic_load(&p);
    printf("INFO ptr_fetch_add old=%p now=%p cells2=%p byte_delta=%ld\n",
           (void *)old, (void *)now, (void *)&cells[2], (long)((char *)now - (char *)old));
    int pass = old == &cells[0] && (char *)now - (char *)old == 2;
    printf("%s ptr_fetch_add_is_bytes\nRESULT %s fails=%d\n",
           pass ? "PASS" : "FAIL", pass ? "PASS" : "FAIL", pass ? 0 : 1);
    return pass ? 0 : 1;
}
