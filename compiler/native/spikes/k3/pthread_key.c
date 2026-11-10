// K3: pthread keys as the portable TLS mechanism (ARCH 7.3). Checks per-thread
// values and that the destructor runs once per thread that set a value.
#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <stdatomic.h>
static pthread_key_t key;
static _Atomic int dtor_calls;
static void dtor(void *p) { atomic_fetch_add(&dtor_calls, 1); free(p); }
static void *w(void *a) {
    long id = (long)a;
    int *v = malloc(sizeof(int)); *v = (int)(id * 10);
    pthread_setspecific(key, v);
    int *got = pthread_getspecific(key);
    return (void *)(long)(got ? *got : -1);
}
int main(void) {
    int fails = 0;
    pthread_key_create(&key, dtor);
    int *mv = malloc(sizeof(int)); *mv = 5; pthread_setspecific(key, mv);
    pthread_t t[4]; void *r[4];
    for (long i = 0; i < 4; i++) pthread_create(&t[i], 0, w, (void *)(i + 1));
    for (int i = 0; i < 4; i++) pthread_join(t[i], &r[i]);
    int ok = *(int *)pthread_getspecific(key) == 5;
    for (int i = 0; i < 4; i++) ok = ok && (long)r[i] == (i + 1) * 10;
    printf("%s pthread_key_per_thread\n", ok ? "PASS" : "FAIL"); if (!ok) fails++;
    // Worker destructors run at thread exit, before join returns on glibc.
    ok = atomic_load(&dtor_calls) == 4;
    printf("%s pthread_key_dtor_workers calls=%d want=4\n", ok ? "PASS" : "FAIL", atomic_load(&dtor_calls));
    if (!ok) fails++;
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
