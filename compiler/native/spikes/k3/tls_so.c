// K3: TLS defined in a shared object. Built with tcc -shared (libbase style).
_Thread_local int tls_flag;
_Thread_local long tls_big = 1234;
static _Thread_local int tls_static = 55;
int get_flag(void) { return tls_flag; }
void set_flag(int v) { tls_flag = v; }
long get_big(void) { return tls_big; }
void set_big(long v) { tls_big = v; }
int get_static(void) { return tls_static; }
void set_static(int v) { tls_static = v; }
void *flag_addr(void) { return &tls_flag; }
