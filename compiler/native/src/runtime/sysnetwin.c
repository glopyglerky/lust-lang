/* Socket intrinsics for Windows (rel-win): not built yet.
   Spliced instead of sysnet.c under _WIN32. tsgo's hot path uses
   no sockets (LSP runs on stdio and named pipes); every call here
   fails with ERROR_NOT_SUPPORTED until the Winsock slice lands.
   A program that opens a socket on Windows therefore differs from
   rustc: this file must gain real bodies before such a program is
   in scope (see REPORT.md). */
#define LUST_NET_NOSUP 50
int32_t lust_sys_net_resolve(const uint8_t *host, uint64_t hostlen, uint64_t port, lust_Vec *out) { (void)host; (void)hostlen; (void)port; const char *t = "name resolution not supported"; lust_vec_reserve(out, 29, 1, 1); memcpy((uint8_t*)out->ptr + out->len, t, 29); out->len += 29; return -1; }
int32_t lust_sys_net_tcp_bind(const uint8_t *rec, uint64_t len, lust_Vec *out) { (void)rec; (void)len; (void)out; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_udp_bind(const uint8_t *rec, uint64_t len, lust_Vec *out) { (void)rec; (void)len; (void)out; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_tcp_connect(const uint8_t *rec, uint64_t len, lust_Vec *out) { (void)rec; (void)len; (void)out; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_tcp_connect_timeout(const uint8_t *rec, uint64_t len, int64_t timeout_us, lust_Vec *out) { (void)rec; (void)len; (void)timeout_us; (void)out; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_connect_fd(int32_t fd, const uint8_t *rec, uint64_t len) { (void)fd; (void)rec; (void)len; return LUST_NET_NOSUP; }
int32_t lust_sys_net_accept(int32_t fd, lust_Vec *out) { (void)fd; (void)out; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_addr_of(int32_t fd, int32_t peer, lust_Vec *out) { (void)fd; (void)peer; (void)out; return LUST_NET_NOSUP; }
int32_t lust_sys_net_shutdown(int32_t fd, int32_t how) { (void)fd; (void)how; return LUST_NET_NOSUP; }
int32_t lust_sys_net_setopt(int32_t fd, int32_t opt, int64_t val) { (void)fd; (void)opt; (void)val; return LUST_NET_NOSUP; }
int64_t lust_sys_net_getopt(int32_t fd, int32_t opt) { (void)fd; (void)opt; return -LUST_NET_NOSUP; }
int64_t lust_sys_net_sendto(int32_t fd, const uint8_t *p, uint64_t len, const uint8_t *rec, uint64_t rlen) { (void)fd; (void)p; (void)len; (void)rec; (void)rlen; return -LUST_NET_NOSUP; }
int64_t lust_sys_net_recvfrom(int32_t fd, uint64_t max, int32_t flags, lust_Vec *out, lust_Vec *addrout) { (void)fd; (void)max; (void)flags; (void)out; (void)addrout; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_unix_pair(lust_Vec *out) { (void)out; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_unix_bind(const uint8_t *p, uint64_t len, int32_t abstract) { (void)p; (void)len; (void)abstract; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_unix_connect(const uint8_t *p, uint64_t len, int32_t abstract) { (void)p; (void)len; (void)abstract; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_unix_accept(int32_t fd, lust_Vec *out) { (void)fd; (void)out; return -LUST_NET_NOSUP; }
int32_t lust_sys_net_unix_addr(int32_t fd, int32_t peer, lust_Vec *out) { (void)fd; (void)peer; (void)out; return LUST_NET_NOSUP; }
