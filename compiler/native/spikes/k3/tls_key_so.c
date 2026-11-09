// K3: the recommended daemon pattern. Same interface as tls_so.c, but the
// storage is a pthread key, not _Thread_local. Built -shared by tcc into a
// libbase-style .so and exercised by tls_linked.c and tls_dlopen.c.
#include <pthread.h>
#include <stdlib.h>
struct tls_state { int flag; long big; int st; };
static pthread_key_t key;
static pthread_once_t once = PTHREAD_ONCE_INIT;
static void make_key(void) { pthread_key_create(&key, free); }
static struct tls_state *state(void) {
    pthread_once(&once, make_key);
    struct tls_state *s = pthread_getspecific(key);
    if (!s) {
        s = calloc(1, sizeof(*s));
        s->big = 1234; s->st = 55;
        pthread_setspecific(key, s);
    }
    return s;
}
int get_flag(void) { return state()->flag; }
void set_flag(int v) { state()->flag = v; }
long get_big(void) { return state()->big; }
void set_big(long v) { state()->big = v; }
int get_static(void) { return state()->st; }
void set_static(int v) { state()->st = v; }
void *flag_addr(void) { return &state()->flag; }
