// K3: exe linked against tls_so (libbase+overlay shape). Also the alias test:
// the exe has its own _Thread_local; a .so access must never read or write it.
#include <stdio.h>
#include <pthread.h>
int get_flag(void); void set_flag(int);
long get_big(void); void set_big(long);
int get_static(void); void set_static(int);
void *flag_addr(void);
_Thread_local long exe_mine = 1234;
static void *w(void *a) { (void)a; set_flag(9); return (void *)(long)get_flag(); }
int main(void) {
    int fails = 0;
    set_flag(3);
    pthread_t t; void *r = 0;
    pthread_create(&t, 0, w, 0); pthread_join(t, &r);
    int ok = get_flag() == 3 && (long)r == 9;
    printf("%s tls_so_per_thread main=%d thread=%ld\n", ok ? "PASS" : "FAIL", get_flag(), (long)r);
    if (!ok) fails++;
    ok = get_big() == 1234 && get_static() == 55;
    printf("%s tls_so_init_values big=%ld static=%d\n", ok ? "PASS" : "FAIL", get_big(), get_static());
    if (!ok) fails++;
    // Alias test (Opus B9): if the .so emits a raw %fs offset with no TPOFF
    // relocation, set_flag/get_flag hit the exe TLS slot at the same offset.
    exe_mine = 1234; set_flag(3);
    int so_after_set = get_flag(); long exe_after_set = exe_mine;
    exe_mine = 77; int so_after_exe = get_flag();
    set_flag(41); long exe_after_so = exe_mine;
    printf("INFO alias so_after_set=%d exe_after_set=%ld so_after_exe_write77=%d exe_after_so_write41=%ld flag_addr=%p exe_addr=%p\n",
           so_after_set, exe_after_set, so_after_exe, exe_after_so, flag_addr(), (void *)&exe_mine);
    ok = so_after_set == 3 && exe_after_set == 1234 && so_after_exe == 3 && exe_after_so == 77
         && flag_addr() != (void *)&exe_mine;
    printf("%s tls_so_no_alias_exe\n", ok ? "PASS" : "FAIL");
    if (!ok) fails++;
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
