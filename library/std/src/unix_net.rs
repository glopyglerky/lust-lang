// `std::os::unix::net` overlay (docs/ARCH.md section 3.1;
// w-cinterop2): UnixStream, UnixListener, and SocketAddr over
// the sys_net_unix intrinsics, plus the `SocketAddrExt` trait
// of `std::os::linux::net` (the same hidden module serves both
// deep paths, src/stdmods.rs). Debug forms follow lust's
// std::net forms (a plain fd field), not std's FileDesc form,
// as with TcpStream. Not here: UnixDatagram. The OwnedFd
// conversions of UnixStream are here (tf-p6 R11).

use std::io;
use std::path::Path;

/// An OS error from its errno.
fn os_err(code: i32) -> io::Error {
    io::Error::from_raw_os_error(code)
}

/// The result of a status that is 0 or an errno.
fn net_status(code: i32) -> io::Result<()> {
    if code == 0 { Ok(()) } else { Err(os_err(code)) }
}

/// Little-endian u64 at byte offset `off` of a sysnet record.
fn rec64(raw: &[u8], off: usize) -> u64 {
    let mut v: u64 = 0;
    let mut i: usize = 0;
    while i < 8 {
        v |= (raw[off + i] as u64) << (8 * i);
        i += 1;
    }
    v
}

/// A timeout duration in whole microseconds (net.rs has the
/// same conversion for the IP sockets).
fn timeout_us(d: core::time::Duration) -> i64 {
    let us = d.as_secs() as i64 * 1_000_000 + (d.subsec_nanos() / 1000) as i64;
    if us == 0 { 1 } else { us }
}

/// Sets one socket timeout option (4 read, 5 write).
fn set_timeout(fd: i32, opt: i32, timeout: Option<core::time::Duration>) -> io::Result<()> {
    let us = match timeout {
        None => 0,
        Some(d) => {
            if d == core::time::Duration::ZERO {
                return Err(io::Error::__lust_adapter_simple_message(io::ErrorKind::InvalidInput, "cannot set a 0 duration timeout"));
            }
            timeout_us(d)
        }
    };
    net_status(core::intrinsics::sys_net_setopt(fd, opt, us))
}

/// Reads one socket timeout option (4 read, 5 write).
fn get_timeout(fd: i32, opt: i32) -> io::Result<Option<core::time::Duration>> {
    let v = core::intrinsics::sys_net_getopt(fd, opt);
    if v < 0 {
        Err(os_err((-v) as i32))
    } else if v == 0 {
        Ok(None)
    } else {
        Ok(Some(core::time::Duration::from_micros(v as u64)))
    }
}

/// One read into `buf`.
fn read_fd_into(fd: i32, buf: &mut [u8]) -> io::Result<usize> {
    let mut tmp: Vec<u8> = Vec::new();
    let n = core::intrinsics::sys_fd_read(fd, buf.len() as u64, &mut tmp);
    if n < 0 {
        return Err(os_err((-n) as i32));
    }
    let n = n as usize;
    let mut i: usize = 0;
    while i < n {
        buf[i] = tmp[i];
        i += 1;
    }
    Ok(n)
}

/// One write of `buf`.
fn write_fd(fd: i32, buf: &[u8]) -> io::Result<usize> {
    let n = core::intrinsics::sys_fd_write(fd, buf);
    if n < 0 { Err(os_err((-n) as i32)) } else { Ok(n as usize) }
}

const KIND_UNNAMED: u8 = 0;
const KIND_PATHNAME: u8 = 1;
const KIND_ABSTRACT: u8 = 2;

/// An address associated with a Unix socket.
#[derive(Clone)]
pub struct SocketAddr {
    kind: u8,
    name: Vec<u8>,
}

/// Decodes one sysnet unix address record.
fn unpack_unix_addr(raw: &[u8]) -> SocketAddr {
    if raw.len() < 8 {
        return SocketAddr { kind: KIND_UNNAMED, name: Vec::new() };
    }
    let kind = rec64(raw, 0) as u8;
    let mut name: Vec<u8> = Vec::new();
    let mut i: usize = 8;
    while i < raw.len() {
        name.push(raw[i]);
        i += 1;
    }
    SocketAddr { kind, name }
}

/// The pathname length error, with std's text.
fn sun_len_err(abstract_name: bool) -> io::Error {
    let what = if abstract_name { "abstract socket name must be shorter than SUN_LEN" } else { "path must be shorter than SUN_LEN" };
    // std's `const_error!`: a static message (its Debug is `Error {
    // kind, message }`, not the `Custom` form).
    io::Error::__lust_adapter_simple_message(io::ErrorKind::InvalidInput, what)
}

/// Checks a pathname against SUN_LEN (108 with the terminator).
fn check_pathname(bytes: &[u8]) -> io::Result<()> {
    if bytes.len() >= 108 { Err(sun_len_err(false)) } else { Ok(()) }
}

impl SocketAddr {
    /// Creates an address from a pathname.
    pub fn from_pathname<P: AsRef<Path>>(path: P) -> io::Result<SocketAddr> {
        let bytes = path.as_ref().as_os_str().as_encoded_bytes();
        check_pathname(bytes)?;
        Ok(SocketAddr { kind: KIND_PATHNAME, name: bytes.to_vec() })
    }

    /// True for an unnamed address.
    pub fn is_unnamed(&self) -> bool {
        self.kind == KIND_UNNAMED
    }

    /// The pathname, for a pathname address.
    pub fn as_pathname(&self) -> Option<&Path> {
        if self.kind != KIND_PATHNAME {
            return None;
        }
        let os: &std::ffi::OsStr = &self.name[..];
        Some(Path::new(os))
    }

    /// The (kind, bytes, abstract flag) a sysnet call takes.
    fn parts(&self) -> (&[u8], i32) {
        match self.kind {
            KIND_ABSTRACT => (&self.name[..], 1),
            _ => (&self.name[..], 0),
        }
    }
}

/// Linux-only extensions to `SocketAddr` (abstract names).
pub trait SocketAddrExt {
    /// Creates an abstract address from a name (without the
    /// leading NUL byte the kernel form carries).
    fn from_abstract_name<N>(name: N) -> io::Result<SocketAddr>
    where
        N: AsRef<[u8]>;
    /// The abstract name, for an abstract address.
    fn as_abstract_name(&self) -> Option<&[u8]>;
}

impl SocketAddrExt for SocketAddr {
    fn from_abstract_name<N>(name: N) -> io::Result<SocketAddr>
    where
        N: AsRef<[u8]>,
    {
        let name = name.as_ref();
        if name.len() > 107 {
            return Err(sun_len_err(true));
        }
        Ok(SocketAddr { kind: KIND_ABSTRACT, name: name.to_vec() })
    }

    fn as_abstract_name(&self) -> Option<&[u8]> {
        if self.kind == KIND_ABSTRACT { Some(&self.name[..]) } else { None }
    }
}

impl core::fmt::Debug for SocketAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            KIND_PATHNAME => match self.as_pathname() {
                Some(p) => write!(f, "{p:?} (pathname)"),
                None => f.write_str("(unnamed)"),
            },
            KIND_ABSTRACT => {
                // std prints the name as a `ByteStr`.
                std::ffi::__lust_bytestr_debug(&self.name, f)?;
                f.write_str(" (abstract)")
            }
            _ => f.write_str("(unnamed)"),
        }
    }
}

/// A Unix stream between two local sockets. Dropping it closes
/// the descriptor.
pub struct UnixStream {
    fd: i32,
}

impl UnixStream {
    /// Connects to the socket at a path.
    pub fn connect<P: AsRef<Path>>(path: P) -> io::Result<UnixStream> {
        let bytes = path.as_ref().as_os_str().as_encoded_bytes();
        check_pathname(bytes)?;
        let fd = core::intrinsics::sys_net_unix_connect(bytes, 0);
        if fd < 0 { Err(os_err(-fd)) } else { Ok(UnixStream { fd }) }
    }

    /// Connects to a socket address.
    pub fn connect_addr(addr: &SocketAddr) -> io::Result<UnixStream> {
        let (bytes, abstract_flag) = addr.parts();
        let fd = core::intrinsics::sys_net_unix_connect(bytes, abstract_flag);
        if fd < 0 { Err(os_err(-fd)) } else { Ok(UnixStream { fd }) }
    }

    /// Creates a connected pair of streams.
    pub fn pair() -> io::Result<(UnixStream, UnixStream)> {
        let mut raw: Vec<u8> = Vec::new();
        let fd = core::intrinsics::sys_net_unix_pair(&mut raw);
        if fd < 0 {
            return Err(os_err(-fd));
        }
        Ok((UnixStream { fd }, UnixStream { fd: rec64(&raw, 0) as i32 }))
    }

    /// The local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        addr_of(self.fd, false)
    }

    /// The peer address.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        addr_of(self.fd, true)
    }

    /// A new handle to the same connection.
    pub fn try_clone(&self) -> io::Result<UnixStream> {
        let fd = core::intrinsics::sys_fd_dup(self.fd);
        if fd < 0 { Err(os_err(-fd)) } else { Ok(UnixStream { fd }) }
    }

    /// Shuts down part of the connection.
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        let h = match how {
            std::net::Shutdown::Read => 0,
            std::net::Shutdown::Write => 1,
            std::net::Shutdown::Both => 2,
        };
        net_status(core::intrinsics::sys_net_shutdown(self.fd, h))
    }

    /// Moves the socket into or out of nonblocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        net_status(core::intrinsics::sys_net_setopt(self.fd, 3, if nonblocking { 1 } else { 0 }))
    }

    /// Sets the read timeout (SO_RCVTIMEO).
    pub fn set_read_timeout(&self, timeout: Option<core::time::Duration>) -> io::Result<()> {
        set_timeout(self.fd, 4, timeout)
    }

    /// The read timeout.
    pub fn read_timeout(&self) -> io::Result<Option<core::time::Duration>> {
        get_timeout(self.fd, 4)
    }

    /// Sets the write timeout (SO_SNDTIMEO).
    pub fn set_write_timeout(&self, timeout: Option<core::time::Duration>) -> io::Result<()> {
        set_timeout(self.fd, 5, timeout)
    }

    /// The write timeout.
    pub fn write_timeout(&self) -> io::Result<Option<core::time::Duration>> {
        get_timeout(self.fd, 5)
    }
}

/// The local or peer unix address of a socket.
fn addr_of(fd: i32, peer: bool) -> io::Result<SocketAddr> {
    let mut raw: Vec<u8> = Vec::new();
    match net_status(core::intrinsics::sys_net_unix_addr(fd, if peer { 1 } else { 0 }, &mut raw)) {
        Ok(()) => Ok(unpack_unix_addr(&raw)),
        Err(e) => Err(e),
    }
}

impl Drop for UnixStream {
    fn drop(&mut self) {
        if self.fd >= 0 {
            let _ = core::intrinsics::sys_fd_close(self.fd);
        }
    }
}

impl From<UnixStream> for std::os::fd::OwnedFd {
    fn from(mut stream: UnixStream) -> std::os::fd::OwnedFd {
        let fd = stream.fd;
        stream.fd = -1;
        <std::os::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(fd)
    }
}

impl From<std::os::fd::OwnedFd> for UnixStream {
    fn from(fd: std::os::fd::OwnedFd) -> UnixStream {
        UnixStream { fd: std::os::fd::IntoRawFd::into_raw_fd(fd) }
    }
}

impl core::fmt::Debug for UnixStream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("UnixStream");
        if let Ok(addr) = self.local_addr() {
            d.field("local", &addr);
        }
        if let Ok(peer) = self.peer_addr() {
            d.field("peer", &peer);
        }
        d.field("fd", &self.fd).finish()
    }
}

impl io::Read for UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read_fd_into(self.fd, buf)
    }
}

impl io::Read for &UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read_fd_into(self.fd, buf)
    }
}

impl io::Write for UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write_fd(self.fd, buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Write for &UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write_fd(self.fd, buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A Unix listener. Dropping it closes the descriptor; the
/// socket file stays, as in std.
pub struct UnixListener {
    fd: i32,
}

impl UnixListener {
    /// Binds and listens at a path.
    pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<UnixListener> {
        let bytes = path.as_ref().as_os_str().as_encoded_bytes();
        check_pathname(bytes)?;
        let fd = core::intrinsics::sys_net_unix_bind(bytes, 0);
        if fd < 0 { Err(os_err(-fd)) } else { Ok(UnixListener { fd }) }
    }

    /// Binds and listens at a socket address.
    pub fn bind_addr(addr: &SocketAddr) -> io::Result<UnixListener> {
        let (bytes, abstract_flag) = addr.parts();
        let fd = core::intrinsics::sys_net_unix_bind(bytes, abstract_flag);
        if fd < 0 { Err(os_err(-fd)) } else { Ok(UnixListener { fd }) }
    }

    /// Accepts a new incoming connection.
    pub fn accept(&self) -> io::Result<(UnixStream, SocketAddr)> {
        let mut raw: Vec<u8> = Vec::new();
        let fd = core::intrinsics::sys_net_unix_accept(self.fd, &mut raw);
        if fd < 0 { Err(os_err(-fd)) } else { Ok((UnixStream { fd }, unpack_unix_addr(&raw))) }
    }

    /// Returns an iterator over incoming connections.
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }

    /// The local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        addr_of(self.fd, false)
    }

    /// Moves the listener into or out of nonblocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        net_status(core::intrinsics::sys_net_setopt(self.fd, 3, if nonblocking { 1 } else { 0 }))
    }
}

impl Drop for UnixListener {
    fn drop(&mut self) {
        if self.fd >= 0 {
            let _ = core::intrinsics::sys_fd_close(self.fd);
        }
    }
}

impl core::fmt::Debug for UnixListener {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("UnixListener");
        if let Ok(addr) = self.local_addr() {
            d.field("local", &addr);
        }
        d.field("fd", &self.fd).finish()
    }
}

/// An iterator over incoming connections to a listener.
pub struct Incoming<'a> {
    listener: &'a UnixListener,
}

impl Iterator for Incoming<'_> {
    type Item = io::Result<UnixStream>;
    fn next(&mut self) -> Option<io::Result<UnixStream>> {
        Some(match self.listener.accept() {
            Ok((s, _)) => Ok(s),
            Err(e) => Err(e),
        })
    }
}

impl core::fmt::Debug for Incoming<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Incoming").finish_non_exhaustive()
    }
}
