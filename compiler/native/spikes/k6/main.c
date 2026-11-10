// K6: semantic tests for one link path. The same binary shape is built by
// every link path in run.sh; a path passes only if this prints RESULT PASS.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include "shim.h"
static int fails;
#define CHECK(name, cond) do { int ok_ = !!(cond); printf("%s %s\n", ok_ ? "PASS" : "FAIL", name); if (!ok_) fails++; } while (0)

static void *tls_worker(void *a) {
    (void)a;
    long before = shim_tls_get();
    shim_tls_set(222);
    long after = shim_tls_get();
    return (void *)(before * 1000 + after);
}
struct cb_env { long sum; long stop_at; };
static int on_value(void *env, long v) {
    struct cb_env *e = env;
    e->sum += v;
    return v == e->stop_at ? 7 : 0;
}
int main(void) {
    setvbuf(stdout, 0, _IOLBF, 0); // keep completed checks visible if a later one aborts
    CHECK("shim_add", shim_add(20, 22) == 42);
    char msg[128] = {0};
    int caught = shim_catch_panic(msg, sizeof msg);
    printf("INFO catch_panic msg='%s'\n", msg);
    CHECK("shim_catch_panic", caught == 1 && strcmp(msg, "shim internal boom 42") == 0);
    shim_tls_set(111);
    pthread_t t; void *r = 0;
    pthread_create(&t, 0, tls_worker, 0); pthread_join(t, &r);
    // worker before=-7 (fresh TLS), after=222; main still 111
    CHECK("shim_tls_per_thread", shim_tls_get() == 111 && (long)r == -7 * 1000 + 222);
    long sum = 0;
    int rc = shim_spawn_sum(4, &sum);
    // threads return (100+i)+1 each: 101+102+103+104 = 410
    printf("INFO spawn_sum rc=%d sum=%ld main_tls=%ld\n", rc, sum, shim_tls_get());
    CHECK("shim_spawn_threads_tls_catch", rc == 0 && sum == 410 && shim_tls_get() == 111);
    struct cb_env e = {0, -1};
    long calls = 0;
    int st = shim_call_callback(on_value, &e, 10, &calls);
    CHECK("shim_callback_all", st == 0 && calls == 10 && e.sum == 45);
    struct cb_env e2 = {0, 3};
    calls = 0;
    st = shim_call_callback(on_value, &e2, 10, &calls);
    CHECK("shim_callback_stop_on_status", st == 7 && calls == 4 && e2.sum == 6);
    unsigned char *buf = shim_alloc_buffer(4096, 0xab);
    CHECK("shim_alloc_check", buf && shim_check_buffer(buf, 4096, 0xab) == 1);
    shim_free_buffer(buf, 4096);
    unsigned char *mine = malloc(1024);
    size_t filled = shim_fill_caller(mine, 1024, 0xcd);
    CHECK("shim_fill_caller_buffer", filled == 1024 && shim_check_buffer(mine, 1024, 0xcd) == 1);
    free(mine);
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
