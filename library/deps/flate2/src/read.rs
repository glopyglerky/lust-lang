// `flate2::read`: the decoders. Each pulls its whole input
// through on the first read (the module comment in lib.rs
// says why) and then serves the decoded bytes; a decode error
// is returned after the bytes decoded before it.

use crate::inflate::{Fail, inflate};
use crate::{adler32, crc32, eof_err, invalid};
use std::io::{self, Read};

/// The zlib and deflate truncated-stream error.
fn incomplete_err() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "incomplete deflate stream")
}

/// One decoded stream held in memory, with a deferred error.
struct Decoded {
    data: Vec<u8>,
    pos: usize,
    err: Option<io::Error>,
}

impl Decoded {
    /// Serves decoded bytes, then the deferred error once.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos < self.data.len() {
            let n = buf.len().min(self.data.len() - self.pos);
            for i in 0..n {
                buf[i] = self.data[self.pos + i];
            }
            self.pos += n;
            return Ok(n);
        }
        match self.err.take() {
            Some(e) => Err(e),
            None => Ok(0),
        }
    }
}

/// A decoder over a reader, decoded lazily by `make` on the
/// first read. Private: upstream has no such public type.
struct Decoder<R> {
    inner: R,
    make: fn(&[u8]) -> Decoded,
    state: Option<Decoded>,
}

impl<R> Decoder<R> {
    fn new(inner: R, make: fn(&[u8]) -> Decoded) -> Decoder<R> {
        Decoder { inner, make, state: None }
    }

    /// The inner reader.
    fn get_ref(&self) -> &R {
        &self.inner
    }

    /// The inner reader, mutably.
    fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.state.is_none() {
            let mut input: Vec<u8> = Vec::new();
            let made = match self.inner.read_to_end(&mut input) {
                Ok(_) => (self.make)(&input),
                Err(e) => Decoded { data: Vec::new(), pos: 0, err: Some(e) },
            };
            self.state = Some(made);
        }
        match self.state.as_mut() {
            Some(s) => s.read(buf),
            None => Ok(0),
        }
    }
}

/// Decodes one gzip member at the start of `input` into `out`.
/// Returns the bytes consumed (header, body, and trailer).
fn gzip_member(input: &[u8], out: &mut Vec<u8>) -> Result<usize, io::Error> {
    if input.len() < 10 {
        // Upstream reads the fixed header whole first, so a
        // short rest is a truncation whatever its bytes are.
        return Err(eof_err());
    }
    if input[0] != 0x1f || input[1] != 0x8b || input[2] != 8 || input[3] & 0xe0 != 0 {
        return Err(invalid("invalid gzip header"));
    }
    let flg = input[3];
    let mut at = 10usize;
    if flg & 4 != 0 {
        if at + 2 > input.len() {
            return Err(eof_err());
        }
        let xlen = (input[at] as usize) | ((input[at + 1] as usize) << 8);
        at += 2 + xlen;
    }
    for flag in [8u8, 16] {
        if flg & flag != 0 {
            while at < input.len() && input[at] != 0 {
                at += 1;
            }
            at += 1;
        }
    }
    if at > input.len() {
        return Err(eof_err());
    }
    if flg & 2 != 0 {
        if at + 2 > input.len() {
            return Err(eof_err());
        }
        let want = (input[at] as u16) | ((input[at + 1] as u16) << 8);
        if want != (crc32(&input[..at], 0) & 0xffff) as u16 {
            return Err(invalid("corrupt gzip stream does not have a matching checksum"));
        }
        at += 2;
    }
    let before = out.len();
    let used = match inflate(&input[at..], out) {
        Ok(u) => u,
        Err(Fail::Short) => return Err(eof_err()),
        Err(Fail::Corrupt) => return Err(invalid("corrupt deflate stream")),
    };
    let trailer = at + used;
    if trailer + 8 > input.len() {
        return Err(eof_err());
    }
    let le32 = |b: &[u8], off: usize| (b[off] as u32) | ((b[off + 1] as u32) << 8) | ((b[off + 2] as u32) << 16) | ((b[off + 3] as u32) << 24);
    let crc = le32(input, trailer);
    let size = le32(input, trailer + 4);
    let body = &out[before..];
    if crc32(body, 0) != crc || body.len() as u32 != size {
        return Err(invalid("corrupt gzip stream does not have a matching checksum"));
    }
    Ok(trailer + 8)
}

/// The whole-input decode of `GzDecoder` (one member).
fn decode_gzip_one(input: &[u8]) -> Decoded {
    let mut data: Vec<u8> = Vec::new();
    let err = match gzip_member(input, &mut data) {
        Ok(_) => None,
        Err(e) => Some(e),
    };
    Decoded { data, pos: 0, err }
}

/// The whole-input decode of `MultiGzDecoder` (all members).
fn decode_gzip_multi(input: &[u8]) -> Decoded {
    let mut data: Vec<u8> = Vec::new();
    let mut at = 0usize;
    loop {
        match gzip_member(&input[at..], &mut data) {
            Ok(used) => {
                at += used;
                if at >= input.len() {
                    return Decoded { data, pos: 0, err: None };
                }
            }
            Err(e) => return Decoded { data, pos: 0, err: Some(e) },
        }
    }
}

/// The whole-input decode of `ZlibDecoder`.
fn decode_zlib(input: &[u8]) -> Decoded {
    let mut data: Vec<u8> = Vec::new();
    let err = decode_zlib_into(input, &mut data).err();
    Decoded { data, pos: 0, err }
}

/// Decodes one zlib stream into `out`.
fn decode_zlib_into(input: &[u8], out: &mut Vec<u8>) -> Result<(), io::Error> {
    if input.len() < 2 {
        return Err(incomplete_err());
    }
    let cmf = input[0];
    let flg = input[1];
    if cmf & 0x0f != 8 || (cmf as u32 * 256 + flg as u32) % 31 != 0 || flg & 0x20 != 0 {
        return Err(invalid("corrupt deflate stream"));
    }
    let used = match inflate(&input[2..], out) {
        Ok(u) => u,
        Err(Fail::Short) => return Err(incomplete_err()),
        Err(Fail::Corrupt) => return Err(invalid("corrupt deflate stream")),
    };
    let trailer = 2 + used;
    if trailer + 4 > input.len() {
        return Err(incomplete_err());
    }
    let want = ((input[trailer] as u32) << 24) | ((input[trailer + 1] as u32) << 16) | ((input[trailer + 2] as u32) << 8) | (input[trailer + 3] as u32);
    if adler32(out, 1) != want {
        return Err(invalid("corrupt deflate stream"));
    }
    Ok(())
}

/// The whole-input decode of `DeflateDecoder` (raw deflate).
fn decode_deflate(input: &[u8]) -> Decoded {
    let mut data: Vec<u8> = Vec::new();
    let err = match inflate(input, &mut data) {
        Ok(_) => None,
        Err(Fail::Short) => Some(incomplete_err()),
        Err(Fail::Corrupt) => Some(invalid("corrupt deflate stream")),
    };
    Decoded { data, pos: 0, err }
}

/// A gzip decoder for one member.
pub struct GzDecoder<R>(Decoder<R>);

impl<R: Read> GzDecoder<R> {
    /// A decoder over `inner`.
    pub fn new(inner: R) -> GzDecoder<R> {
        GzDecoder(Decoder::new(inner, decode_gzip_one))
    }
}

// The accessors need no `Read` bound, as upstream.
impl<R> GzDecoder<R> {
    /// The inner reader.
    pub fn get_ref(&self) -> &R {
        self.0.get_ref()
    }

    /// The inner reader, mutably.
    pub fn get_mut(&mut self) -> &mut R {
        self.0.get_mut()
    }
}

impl<R: Read> Read for GzDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

/// A gzip decoder for concatenated members.
pub struct MultiGzDecoder<R>(Decoder<R>);

impl<R: Read> MultiGzDecoder<R> {
    /// A decoder over `inner`.
    pub fn new(inner: R) -> MultiGzDecoder<R> {
        MultiGzDecoder(Decoder::new(inner, decode_gzip_multi))
    }
}

// The accessors need no `Read` bound, as upstream.
impl<R> MultiGzDecoder<R> {
    /// The inner reader.
    pub fn get_ref(&self) -> &R {
        self.0.get_ref()
    }

    /// The inner reader, mutably.
    pub fn get_mut(&mut self) -> &mut R {
        self.0.get_mut()
    }
}

impl<R: Read> Read for MultiGzDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

/// A zlib decoder.
pub struct ZlibDecoder<R>(Decoder<R>);

impl<R: Read> ZlibDecoder<R> {
    /// A decoder over `inner`.
    pub fn new(inner: R) -> ZlibDecoder<R> {
        ZlibDecoder(Decoder::new(inner, decode_zlib))
    }
}

// The accessors need no `Read` bound, as upstream.
impl<R> ZlibDecoder<R> {
    /// The inner reader.
    pub fn get_ref(&self) -> &R {
        self.0.get_ref()
    }

    /// The inner reader, mutably.
    pub fn get_mut(&mut self) -> &mut R {
        self.0.get_mut()
    }
}

impl<R: Read> Read for ZlibDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

/// A raw deflate decoder.
pub struct DeflateDecoder<R>(Decoder<R>);

impl<R: Read> DeflateDecoder<R> {
    /// A decoder over `inner`.
    pub fn new(inner: R) -> DeflateDecoder<R> {
        DeflateDecoder(Decoder::new(inner, decode_deflate))
    }
}

// The accessors need no `Read` bound, as upstream.
impl<R> DeflateDecoder<R> {
    /// The inner reader.
    pub fn get_ref(&self) -> &R {
        self.0.get_ref()
    }

    /// The inner reader, mutably.
    pub fn get_mut(&mut self) -> &mut R {
        self.0.get_mut()
    }
}

impl<R: Read> Read for DeflateDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}
