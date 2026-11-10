// K3: _Thread_local in an executable only (no .so). Prints PASS/FAIL lines.
#include <stdio.h>
#include <pthread.h>
_Thread_local int mine = 5;
__thread long mine2 = 7;
static _Thread_local int mine_static = 11;
static void *w(void *a) {
    (void)a; int before = mine; mine = 99; mine2 = 88; mine_static = 77;
    return (void *)(long)(before * 1000000 + mine * 10000 + (int)mine2 * 100 + mine_static);
}
int main(void) {
    int fails = 0;
    pthread_t t; void *r = 0;
    pthread_create(&t, 0, w, 0); pthread_join(t, &r);
    long packed = (long)r;
    int ok = mine == 5 && mine2 == 7 && mine_static == 11;
    printf("%s tls_exe_main_unchanged mine=%d mine2=%ld static=%d\n", ok ? "PASS" : "FAIL", mine, mine2, mine_static);
    if (!ok) fails++;
    ok = packed == 5L * 1000000 + 99 * 10000 + 88 * 100 + 77;
    printf("%s tls_exe_worker_init_and_write packed=%ld\n", ok ? "PASS" : "FAIL", packed);
    if (!ok) fails++;
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
