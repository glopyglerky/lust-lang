// K6: dlopen the Rust cdylib from a tcc executable. Arg 1 is the .so path.
#include <stdio.h>
#include <string.h>
#include <dlfcn.h>
#include "shim.h"
int main(int argc, char **argv) {
    void *h = dlopen(argc > 1 ? argv[1] : "./libshim.so", RTLD_NOW);
    if (!h) { printf("FAIL dlopen_open %s\nRESULT FAIL fails=1\n", dlerror()); return 1; }
    printf("PASS dlopen_open\n");
    int fails = 0;
    long (*add)(long, long) = (long (*)(long, long))dlsym(h, "shim_add");
    int (*catchp)(char *, size_t) = (int (*)(char *, size_t))dlsym(h, "shim_catch_panic");
    int (*spawn)(unsigned, long *) = (int (*)(unsigned, long *))dlsym(h, "shim_spawn_sum");
    if (!add || !catchp || !spawn) { printf("FAIL dlopen_sym\nRESULT FAIL fails=1\n"); return 1; }
    int ok = add(20, 22) == 42;
    printf("%s dlopen_add\n", ok ? "PASS" : "FAIL"); if (!ok) fails++;
    char msg[128] = {0};
    ok = catchp(msg, sizeof msg) == 1 && strcmp(msg, "shim internal boom 42") == 0;
    printf("%s dlopen_catch_panic msg='%s'\n", ok ? "PASS" : "FAIL", msg); if (!ok) fails++;
    long sum = 0;
    ok = spawn(4, &sum) == 0 && sum == 410;
    printf("%s dlopen_spawn sum=%ld\n", ok ? "PASS" : "FAIL", sum); if (!ok) fails++;
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
