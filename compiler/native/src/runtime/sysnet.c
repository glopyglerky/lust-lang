#ifdef _WIN32
#include "sysnetwin.c"
#else
/* Bodies of the socket intrinsics declared in sysnet.h
   (w-cinterop). */
#include <poll.h>
#include <time.h>
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

/* glibc exposes accept4 only under _GNU_SOURCE, which this unit does not set (as with
   pipe2 in sysio.c). gcc 14 makes the implicit declaration an error. Darwin has
   neither accept4 nor the SOCK_CLOEXEC/SOCK_NONBLOCK type flags: the flags are
   defined away in the create calls and set with fcntl right after, through
   lust_sock_after_create (not atomic; a fork+exec in another thread in between
   could inherit the fd for a moment, as with pipe2 in sysio.c). */
#ifdef __APPLE__
#ifndef SOCK_CLOEXEC
#define SOCK_CLOEXEC 0
#endif
#ifndef SOCK_NONBLOCK
#define SOCK_NONBLOCK 0
#endif
static void lust_sock_after_create(int fd, int nonblock) {
    if (fd < 0) return;
    fcntl(fd, F_SETFD, FD_CLOEXEC);
    if (nonblock) {
        int fl = fcntl(fd, F_GETFL, 0);
        if (fl >= 0) fcntl(fd, F_SETFL, fl | O_NONBLOCK);
    }
}
static int lust_accept_cloexec(int fd, struct sockaddr *addr, socklen_t *len) {
    int n = accept(fd, addr, len);
    lust_sock_after_create(n, 0);
    return n;
}
#else
int accept4(int fd, struct sockaddr *addr, socklen_t *len, int flags);
static void lust_sock_after_create(int fd, int nonblock) {
    (void)fd;
    (void)nonblock;
}
static int lust_accept_cloexec(int fd, struct sockaddr *addr, socklen_t *len) {
    return accept4(fd, addr, len, SOCK_CLOEXEC);
}
#endif

/* The family of a socket, for the TTL option: SO_DOMAIN on Linux
   (std's way); Darwin has no SO_DOMAIN, and getsockname reports the
   family of a bound or unbound socket there. 0 with errno set when
   neither works. */
static int lust_sock_family(int fd) {
#ifdef __APPLE__
    struct sockaddr_storage ss;
    socklen_t ll = sizeof ss;
    if (getsockname(fd, (struct sockaddr *)&ss, &ll) != 0) return 0;
    return ss.ss_family;
#else
    int fam = 0;
    socklen_t ll = sizeof fam;
    if (getsockopt(fd, SOL_SOCKET, SO_DOMAIN, &fam, &ll) != 0) return 0;
    return fam;
#endif
}

/* Appends `n` bytes to a Vec<u8> (the sysio.c shape). */
static void lust_sysnet_push(lust_Vec *out, const void *p, uint64_t n) {
    if (n == 0) return;
    lust_vec_reserve(out, n, 1, 1);
    memcpy((uint8_t *)out->ptr + out->len, p, (size_t)n);
    out->len += n;
}

static void lust_sysnet_put64(lust_Vec *out, uint64_t v) {
    uint8_t b[8];
    for (int i = 0; i < 8; i++) b[i] = (uint8_t)(v >> (8 * i));
    lust_sysnet_push(out, b, 8);
}

static uint64_t lust_sysnet_get64(const uint8_t *p) {
    uint64_t v = 0;
    for (int i = 0; i < 8; i++) v |= (uint64_t)p[i] << (8 * i);
    return v;
}

/* One sockaddr as an address record appended to `out`. */
static void lust_sysnet_push_addr(lust_Vec *out, const struct sockaddr *sa) {
    uint8_t bytes[16];
    memset(bytes, 0, sizeof bytes);
    if (sa->sa_family == AF_INET) {
        const struct sockaddr_in *s = (const struct sockaddr_in *)sa;
        memcpy(bytes, &s->sin_addr, 4);
        lust_sysnet_put64(out, 4);
        lust_sysnet_put64(out, ntohs(s->sin_port));
        lust_sysnet_put64(out, lust_sysnet_get64(bytes));
        lust_sysnet_put64(out, lust_sysnet_get64(bytes + 8));
        lust_sysnet_put64(out, 0);
        lust_sysnet_put64(out, 0);
    } else if (sa->sa_family == AF_INET6) {
        const struct sockaddr_in6 *s = (const struct sockaddr_in6 *)sa;
        memcpy(bytes, &s->sin6_addr, 16);
        lust_sysnet_put64(out, 6);
        lust_sysnet_put64(out, ntohs(s->sin6_port));
        lust_sysnet_put64(out, lust_sysnet_get64(bytes));
        lust_sysnet_put64(out, lust_sysnet_get64(bytes + 8));
        lust_sysnet_put64(out, s->sin6_flowinfo);
        lust_sysnet_put64(out, s->sin6_scope_id);
    }
}

/* An address record as a sockaddr. Returns the length, or 0 for a
   bad record. */
static socklen_t lust_sysnet_read_addr(const uint8_t *rec, uint64_t len, struct sockaddr_storage *ss) {
    if (len < 48) return 0;
    uint64_t fam = lust_sysnet_get64(rec);
    uint16_t port = (uint16_t)lust_sysnet_get64(rec + 8);
    uint8_t bytes[16];
    for (int i = 0; i < 8; i++) bytes[i] = rec[16 + i];
    for (int i = 0; i < 8; i++) bytes[8 + i] = rec[24 + i];
    memset(ss, 0, sizeof *ss);
    if (fam == 4) {
        struct sockaddr_in *s = (struct sockaddr_in *)ss;
        s->sin_family = AF_INET;
        s->sin_port = htons(port);
        memcpy(&s->sin_addr, bytes, 4);
        return sizeof *s;
    }
    if (fam == 6) {
        struct sockaddr_in6 *s = (struct sockaddr_in6 *)ss;
        s->sin6_family = AF_INET6;
        s->sin6_port = htons(port);
        memcpy(&s->sin6_addr, bytes, 16);
        s->sin6_flowinfo = (uint32_t)lust_sysnet_get64(rec + 32);
        s->sin6_scope_id = (uint32_t)lust_sysnet_get64(rec + 40);
        return sizeof *s;
    }
    return 0;
}

static int lust_sysnet_family(const uint8_t *rec, uint64_t len) {
    if (len < 48) return -1;
    uint64_t fam = lust_sysnet_get64(rec);
    return fam == 4 ? AF_INET : fam == 6 ? AF_INET6 : -1;
}

/* Dual stack on a fresh v6 socket, as std binds on Unix. */
static void lust_sysnet_v6only_off(int fd, int fam) {
    if (fam != AF_INET6) return;
    int off = 0;
    setsockopt(fd, IPPROTO_IPV6, IPV6_V6ONLY, &off, sizeof off);
}

int32_t lust_sys_net_resolve(const uint8_t *host, uint64_t hostlen, uint64_t port, lust_Vec *out) {
    char *name = malloc((size_t)hostlen + 1);
    if (!name) lust_oom();
    memcpy(name, host, (size_t)hostlen);
    name[hostlen] = 0;
    if (memchr(name, 0, (size_t)hostlen)) {
        const char *msg = gai_strerror(EAI_NONAME);
        lust_sysnet_push(out, msg, strlen(msg));
        free(name);
        return -1;
    }
    struct addrinfo hints;
    memset(&hints, 0, sizeof hints);
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    struct addrinfo *res = 0;
    int rc = getaddrinfo(name, 0, &hints, &res);
    free(name);
    if (rc != 0) {
        const char *msg = gai_strerror(rc);
        lust_sysnet_push(out, msg, strlen(msg));
        return -1;
    }
    int32_t n = 0;
    for (struct addrinfo *ai = res; ai; ai = ai->ai_next) {
        if (ai->ai_family == AF_INET) {
            ((struct sockaddr_in *)ai->ai_addr)->sin_port = htons((uint16_t)port);
        } else if (ai->ai_family == AF_INET6) {
            ((struct sockaddr_in6 *)ai->ai_addr)->sin6_port = htons((uint16_t)port);
        } else {
            continue;
        }
        lust_sysnet_push_addr(out, ai->ai_addr);
        n++;
    }
    freeaddrinfo(res);
    return n;
}

static int32_t lust_sysnet_bind(const uint8_t *rec, uint64_t len, lust_Vec *out, int type, int listen_backlog) {
    int fam = lust_sysnet_family(rec, len);
    if (fam < 0) return -EINVAL;
    int fd = socket(fam, type | SOCK_CLOEXEC, 0);
    if (fd < 0) return -errno;
    lust_sock_after_create(fd, 0);
    if (listen_backlog >= 0) {
        int on = 1;
        setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &on, sizeof on);
    }
    lust_sysnet_v6only_off(fd, fam);
    struct sockaddr_storage ss;
    socklen_t sl = lust_sysnet_read_addr(rec, len, &ss);
    if (bind(fd, (struct sockaddr *)&ss, sl) != 0) {
        int e = errno;
        close(fd);
        return -e;
    }
    if (listen_backlog >= 0 && listen(fd, listen_backlog) != 0) {
        int e = errno;
        close(fd);
        return -e;
    }
    struct sockaddr_storage local;
    socklen_t ll = sizeof local;
    if (getsockname(fd, (struct sockaddr *)&local, &ll) == 0) lust_sysnet_push_addr(out, (struct sockaddr *)&local);
    return fd;
}

int32_t lust_sys_net_tcp_bind(const uint8_t *rec, uint64_t len, lust_Vec *out) {
    return lust_sysnet_bind(rec, len, out, SOCK_STREAM, 128);
}

int32_t lust_sys_net_udp_bind(const uint8_t *rec, uint64_t len, lust_Vec *out) {
    return lust_sysnet_bind(rec, len, out, SOCK_DGRAM, -1);
}

int32_t lust_sys_net_tcp_connect(const uint8_t *rec, uint64_t len, lust_Vec *out) {
    int fam = lust_sysnet_family(rec, len);
    if (fam < 0) return -EINVAL;
    int fd = socket(fam, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) return -errno;
    lust_sock_after_create(fd, 0);
    struct sockaddr_storage ss;
    socklen_t sl = lust_sysnet_read_addr(rec, len, &ss);
    if (connect(fd, (struct sockaddr *)&ss, sl) != 0) {
        int e = errno;
        close(fd);
        return -e;
    }
    struct sockaddr_storage local;
    socklen_t ll = sizeof local;
    if (getsockname(fd, (struct sockaddr *)&local, &ll) == 0) lust_sysnet_push_addr(out, (struct sockaddr *)&local);
    return fd;
}

/* connect with a timeout in microseconds: a nonblocking connect
   and a poll, as std's connect_timeout does. Returns the fd in
   blocking mode, -errno, or -10000 when the poll itself timed
   out (std reports that as its own TimedOut error, not an OS
   error). EINTR in poll gives back the time left. */
int32_t lust_sys_net_tcp_connect_timeout(const uint8_t *rec, uint64_t len, int64_t timeout_us, lust_Vec *out) {
    int fam = lust_sysnet_family(rec, len);
    if (fam < 0) return -EINVAL;
    int fd = socket(fam, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0);
    if (fd < 0) return -errno;
    lust_sock_after_create(fd, 1);
    struct sockaddr_storage ss;
    socklen_t sl = lust_sysnet_read_addr(rec, len, &ss);
    if (connect(fd, (struct sockaddr *)&ss, sl) != 0) {
        if (errno != EINPROGRESS) {
            int e = errno;
            close(fd);
            return -e;
        }
        struct timespec start;
        clock_gettime(CLOCK_MONOTONIC, &start);
        int64_t left = timeout_us;
        for (;;) {
            struct pollfd pfd;
            pfd.fd = fd;
            pfd.events = POLLOUT;
            pfd.revents = 0;
            int ms = left >= 1000 ? (int)((left + 999) / 1000 > INT32_MAX ? INT32_MAX : (left + 999) / 1000) : 1;
            int r = poll(&pfd, 1, ms);
            if (r > 0) break;
            if (r == 0) {
                close(fd);
                return -10000;
            }
            if (errno != EINTR) {
                int e = errno;
                close(fd);
                return -e;
            }
            struct timespec now;
            clock_gettime(CLOCK_MONOTONIC, &now);
            int64_t used = (int64_t)(now.tv_sec - start.tv_sec) * 1000000 + (now.tv_nsec - start.tv_nsec) / 1000;
            if (used >= timeout_us) {
                close(fd);
                return -10000;
            }
            left = timeout_us - used;
        }
        int err = 0;
        socklen_t el = sizeof err;
        if (getsockopt(fd, SOL_SOCKET, SO_ERROR, &err, &el) != 0) err = errno;
        if (err != 0) {
            close(fd);
            return -err;
        }
    }
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags >= 0) fcntl(fd, F_SETFL, flags & ~O_NONBLOCK);
    struct sockaddr_storage local;
    socklen_t ll = sizeof local;
    if (getsockname(fd, (struct sockaddr *)&local, &ll) == 0) lust_sysnet_push_addr(out, (struct sockaddr *)&local);
    return fd;
}

int32_t lust_sys_net_connect_fd(int32_t fd, const uint8_t *rec, uint64_t len) {
    struct sockaddr_storage ss;
    socklen_t sl = lust_sysnet_read_addr(rec, len, &ss);
    if (sl == 0) return EINVAL;
    return connect(fd, (struct sockaddr *)&ss, sl) == 0 ? 0 : errno;
}

int32_t lust_sys_net_accept(int32_t fd, lust_Vec *out) {
    struct sockaddr_storage ss;
    socklen_t ll = sizeof ss;
    int n = lust_accept_cloexec(fd, (struct sockaddr *)&ss, &ll);
    if (n < 0) return -errno;
    lust_sysnet_push_addr(out, (struct sockaddr *)&ss);
    return n;
}

int32_t lust_sys_net_addr_of(int32_t fd, int32_t peer, lust_Vec *out) {
    struct sockaddr_storage ss;
    socklen_t ll = sizeof ss;
    int rc = peer ? getpeername(fd, (struct sockaddr *)&ss, &ll) : getsockname(fd, (struct sockaddr *)&ss, &ll);
    if (rc != 0) return errno;
    lust_sysnet_push_addr(out, (struct sockaddr *)&ss);
    return 0;
}

int32_t lust_sys_net_shutdown(int32_t fd, int32_t how) {
    return shutdown(fd, how == 0 ? SHUT_RD : how == 1 ? SHUT_WR : SHUT_RDWR) == 0 ? 0 : errno;
}

int32_t lust_sys_net_setopt(int32_t fd, int32_t opt, int64_t val) {
    int v = (int)val;
    switch (opt) {
        case 0: return setsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &v, sizeof v) == 0 ? 0 : errno;
        case 1: {
            /* std checks the ttl it read back on Linux happens in the
               Rust side only for values it sets; the syscall is plain. */
            int fam = lust_sock_family(fd);
            if (fam == 0) return errno;
            if (fam == AF_INET6) return setsockopt(fd, IPPROTO_IPV6, IPV6_UNICAST_HOPS, &v, sizeof v) == 0 ? 0 : errno;
            return setsockopt(fd, IPPROTO_IP, IP_TTL, &v, sizeof v) == 0 ? 0 : errno;
        }
        case 2: return setsockopt(fd, SOL_SOCKET, SO_BROADCAST, &v, sizeof v) == 0 ? 0 : errno;
        case 3: {
            int flags = fcntl(fd, F_GETFL, 0);
            if (flags < 0) return errno;
            return fcntl(fd, F_SETFL, val ? flags | O_NONBLOCK : flags & ~O_NONBLOCK) == 0 ? 0 : errno;
        }
        case 4:
        case 5: {
            struct timeval tv;
            tv.tv_sec = (time_t)(val / 1000000);
            tv.tv_usec = (suseconds_t)(val % 1000000);
            return setsockopt(fd, SOL_SOCKET, opt == 4 ? SO_RCVTIMEO : SO_SNDTIMEO, &tv, sizeof tv) == 0 ? 0 : errno;
        }
        default: return EINVAL;
    }
}

int64_t lust_sys_net_getopt(int32_t fd, int32_t opt) {
    int v = 0;
    socklen_t ll = sizeof v;
    switch (opt) {
        case 0: return getsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &v, &ll) == 0 ? v : -(int64_t)errno;
        case 1: {
            int fam = lust_sock_family(fd);
            if (fam == 0) return -(int64_t)errno;
            if (fam == AF_INET6) return getsockopt(fd, IPPROTO_IPV6, IPV6_UNICAST_HOPS, &v, &ll) == 0 ? v : -(int64_t)errno;
            return getsockopt(fd, IPPROTO_IP, IP_TTL, &v, &ll) == 0 ? v : -(int64_t)errno;
        }
        case 2: return getsockopt(fd, SOL_SOCKET, SO_BROADCAST, &v, &ll) == 0 ? v : -(int64_t)errno;
        case 3: {
            int flags = fcntl(fd, F_GETFL, 0);
            return flags < 0 ? -(int64_t)errno : (flags & O_NONBLOCK) != 0;
        }
        case 4:
        case 5: {
            struct timeval tv;
            socklen_t tl = sizeof tv;
            if (getsockopt(fd, SOL_SOCKET, opt == 4 ? SO_RCVTIMEO : SO_SNDTIMEO, &tv, &tl) != 0) return -(int64_t)errno;
            return (int64_t)tv.tv_sec * 1000000 + (int64_t)tv.tv_usec;
        }
        default: return -(int64_t)EINVAL;
    }
}

int64_t lust_sys_net_sendto(int32_t fd, const uint8_t *p, uint64_t len, const uint8_t *rec, uint64_t rlen) {
    ssize_t n;
    if (rlen == 0) {
        n = send(fd, p, (size_t)len, MSG_NOSIGNAL);
    } else {
        struct sockaddr_storage ss;
        socklen_t sl = lust_sysnet_read_addr(rec, rlen, &ss);
        if (sl == 0) return -(int64_t)EINVAL;
        n = sendto(fd, p, (size_t)len, MSG_NOSIGNAL, (struct sockaddr *)&ss, sl);
    }
    return n < 0 ? -(int64_t)errno : (int64_t)n;
}

int64_t lust_sys_net_recvfrom(int32_t fd, uint64_t max, int32_t flags, lust_Vec *out, lust_Vec *addrout) {
    if (max == 0) return 0;
    uint8_t *tmp = malloc((size_t)max);
    if (!tmp) lust_oom();
    struct sockaddr_storage ss;
    socklen_t ll = sizeof ss;
    ssize_t n = recvfrom(fd, tmp, (size_t)max, flags & 1 ? MSG_PEEK : 0, (struct sockaddr *)&ss, &ll);
    if (n < 0) {
        int e = errno;
        free(tmp);
        return -(int64_t)e;
    }
    lust_sysnet_push(out, tmp, (uint64_t)n);
    free(tmp);
    lust_sysnet_push_addr(addrout, (struct sockaddr *)&ss);
    return (int64_t)n;
}

/* ---- Unix domain sockets (sysnet.h has the record forms) ---- */

/* One sockaddr_un and its length from path bytes + abstract flag.
   The Rust side checked the SUN_LEN limits. */
static socklen_t lust_sysnet_unix_addr(const uint8_t *p, uint64_t len, int32_t abstract, struct sockaddr_un *un) {
    memset(un, 0, sizeof *un);
    un->sun_family = AF_UNIX;
    socklen_t sl;
    if (abstract) {
        memcpy(un->sun_path + 1, p, (size_t)len);
        sl = (socklen_t)(offsetof(struct sockaddr_un, sun_path) + 1 + len);
    } else {
        memcpy(un->sun_path, p, (size_t)len);
        sl = (socklen_t)(offsetof(struct sockaddr_un, sun_path) + len + 1);
    }
#ifdef __APPLE__
    /* Darwin's sockaddr_un carries its length in sun_len (its first
       byte, before sun_family); the byte count here also covers that
       byte on Darwin, where sa_family_t is one byte. */
    un->sun_len = (unsigned char)sl;
#endif
    return sl;
}

/* One unix address record out: kind u64 then the name bytes. */
static void lust_sysnet_push_unix_addr(lust_Vec *out, const struct sockaddr_un *un, socklen_t ll) {
    uint64_t off = offsetof(struct sockaddr_un, sun_path);
    if (ll <= off) {
        lust_sysnet_put64(out, 0);
        return;
    }
    uint64_t n = ll - off;
    if (un->sun_path[0] == 0) {
        if (n <= 1) {
            lust_sysnet_put64(out, 0);
            return;
        }
        lust_sysnet_put64(out, 2);
        lust_sysnet_push(out, un->sun_path + 1, n - 1);
        return;
    }
    uint64_t sl = 0;
    while (sl < n && un->sun_path[sl] != 0) sl++;
    lust_sysnet_put64(out, 1);
    lust_sysnet_push(out, un->sun_path, sl);
}

int32_t lust_sys_net_unix_pair(lust_Vec *out) {
    int fds[2];
    if (socketpair(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0, fds) != 0) return -errno;
    lust_sock_after_create(fds[0], 0);
    lust_sock_after_create(fds[1], 0);
    lust_sysnet_put64(out, (uint64_t)(uint32_t)fds[1]);
    return fds[0];
}

int32_t lust_sys_net_unix_bind(const uint8_t *p, uint64_t len, int32_t abstract) {
    int fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) return -errno;
    lust_sock_after_create(fd, 0);
    struct sockaddr_un un;
    socklen_t sl = lust_sysnet_unix_addr(p, len, abstract, &un);
    if (bind(fd, (struct sockaddr *)&un, sl) != 0 || listen(fd, 128) != 0) {
        int e = errno;
        close(fd);
        return -e;
    }
    return fd;
}

int32_t lust_sys_net_unix_connect(const uint8_t *p, uint64_t len, int32_t abstract) {
    int fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) return -errno;
    lust_sock_after_create(fd, 0);
    struct sockaddr_un un;
    socklen_t sl = lust_sysnet_unix_addr(p, len, abstract, &un);
    if (connect(fd, (struct sockaddr *)&un, sl) != 0) {
        int e = errno;
        close(fd);
        return -e;
    }
    return fd;
}

int32_t lust_sys_net_unix_accept(int32_t fd, lust_Vec *out) {
    struct sockaddr_un un;
    socklen_t ll = sizeof un;
    int n = lust_accept_cloexec(fd, (struct sockaddr *)&un, &ll);
    if (n < 0) return -errno;
    lust_sysnet_push_unix_addr(out, &un, ll);
    return n;
}

int32_t lust_sys_net_unix_addr(int32_t fd, int32_t peer, lust_Vec *out) {
    struct sockaddr_un un;
    socklen_t ll = sizeof un;
    int rc = peer ? getpeername(fd, (struct sockaddr *)&un, &ll) : getsockname(fd, (struct sockaddr *)&un, &ll);
    if (rc != 0) return errno;
    lust_sysnet_push_unix_addr(out, &un, ll);
    return 0;
}

#endif
