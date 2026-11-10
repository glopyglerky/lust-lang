// K6: like main.c but never triggers a Rust panic. Isolates whether the
// tcc+staticlib path fails only at unwind, or also at TLS/threads/callbacks.
#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include "shim.h"
static int fails;
#define CHECK(name, cond) do { int ok_ = !!(cond); printf("%s %s\n", ok_ ? "PASS" : "FAIL", name); if (!ok_) fails++; } while (0)
static void *tls_worker(void *a) {
    (void)a;
    long before = shim_tls_get();
    shim_tls_set(222);
    return (void *)(before * 1000 + shim_tls_get());
}
struct cb_env { long sum; };
static int on_value(void *env, long v) { ((struct cb_env *)env)->sum += v; return 0; }
int main(void) {
    setvbuf(stdout, 0, _IOLBF, 0); // keep completed checks visible if a later one aborts
    CHECK("shim_add", shim_add(20, 22) == 42);
    shim_tls_set(111);
    pthread_t t; void *r = 0;
    pthread_create(&t, 0, tls_worker, 0); pthread_join(t, &r);
    CHECK("shim_tls_per_thread", shim_tls_get() == 111 && (long)r == -7 * 1000 + 222);
    long sum = 0;
    int rc = shim_spawn_sum_nopanic(4, &sum);
    printf("INFO spawn_sum_nopanic rc=%d sum=%ld main_tls=%ld\n", rc, sum, shim_tls_get());
    CHECK("shim_spawn_threads_nopanic", rc == 0 && sum == 406 && shim_tls_get() == 111);
    struct cb_env e = {0}; long calls = 0;
    int st = shim_call_callback(on_value, &e, 10, &calls);
    CHECK("shim_callback_all", st == 0 && calls == 10 && e.sum == 45);
    unsigned char *buf = shim_alloc_buffer(4096, 0xab);
    CHECK("shim_alloc_check", buf && shim_check_buffer(buf, 4096, 0xab) == 1);
    shim_free_buffer(buf, 4096);
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
