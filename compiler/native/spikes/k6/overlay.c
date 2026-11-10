// K6: overlay executable unit. Links libbase.so the way src/tcc.rs
// link_with_base does (-L dir -lbase -Wl,-rpath=dir).
#include <stdio.h>
long base_compute(long x);
int main(void) {
    long v = base_compute(32); // 32 + 1000 + 410 = 1442
    int ok = v == 1442;
    printf("INFO overlay base_compute=%ld\n%s overlay_base_shim\nRESULT %s fails=%d\n",
           v, ok ? "PASS" : "FAIL", ok ? "PASS" : "FAIL", ok ? 0 : 1);
    return ok ? 0 : 1;
}
