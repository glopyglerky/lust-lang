// K3: dlopen a tcc-built .so that defines TLS. Arg 1 is the .so path.
#include <stdio.h>
#include <dlfcn.h>
#include <pthread.h>
static int (*g)(void); static void (*s)(int);
static void *w(void *a) { (void)a; s(9); return (void *)(long)g(); }
int main(int argc, char **argv) {
    void *h = dlopen(argc > 1 ? argv[1] : "./libtlsso.so", RTLD_NOW);
    if (!h) { printf("FAIL tls_dlopen_open %s\nRESULT FAIL fails=1\n", dlerror()); return 1; }
    printf("PASS tls_dlopen_open\n");
    g = (int (*)(void))dlsym(h, "get_flag"); s = (void (*)(int))dlsym(h, "set_flag");
    if (!g || !s) { printf("FAIL tls_dlopen_sym\nRESULT FAIL fails=1\n"); return 1; }
    s(3); pthread_t t; void *r = 0;
    pthread_create(&t, 0, w, 0); pthread_join(t, &r);
    int ok = g() == 3 && (long)r == 9;
    printf("%s tls_dlopen_per_thread main=%d thread=%ld\n", ok ? "PASS" : "FAIL", g(), (long)r);
    printf("RESULT %s fails=%d\n", ok ? "PASS" : "FAIL", ok ? 0 : 1);
    return ok ? 0 : 1;
}
