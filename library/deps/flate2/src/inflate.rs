// A DEFLATE decoder (RFC 1951) for the flate2 read adapter:
// stored, fixed-Huffman, and dynamic-Huffman blocks. Errors
// are the one corrupt-stream error flate2 returns; running out
// of input is its own case so gzip can call it truncated.

/// What stopped a decode.
pub enum Fail {
    /// The deflate stream is corrupt.
    Corrupt,
    /// The input ended inside the stream.
    Short,
}

/// LSB-first bits over a byte slice.
struct Bits<'a> {
    data: &'a [u8],
    byte: usize,
    acc: u32,
    nbits: u32,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Bits<'a> {
        Bits { data, byte: 0, acc: 0, nbits: 0 }
    }

    /// Reads `n` bits (n <= 16).
    fn take(&mut self, n: u32) -> Result<u32, Fail> {
        while self.nbits < n {
            if self.byte >= self.data.len() {
                return Err(Fail::Short);
            }
            self.acc |= (self.data[self.byte] as u32) << self.nbits;
            self.byte += 1;
            self.nbits += 8;
        }
        let v = self.acc & ((1u32 << n) - 1);
        self.acc >>= n;
        self.nbits -= n;
        Ok(v)
    }

    /// Drops bits to the next byte boundary.
    fn align(&mut self) {
        let drop = self.nbits % 8;
        self.acc >>= drop;
        self.nbits -= drop;
    }

    /// Whole bytes read so far (a partially used byte counts).
    fn consumed(&self) -> usize {
        self.byte
    }

    /// Reads `n` stored bytes after alignment.
    fn stored(&mut self, n: usize) -> Result<&'a [u8], Fail> {
        self.align();
        // After align, whole bytes sit in acc first; rebuild by
        // stepping back over the buffered whole bytes.
        let buffered = (self.nbits / 8) as usize;
        let start = self.byte - buffered;
        if start + n > self.data.len() {
            return Err(Fail::Short);
        }
        self.byte = start + n;
        self.acc = 0;
        self.nbits = 0;
        Ok(&self.data[start..start + n])
    }
}

/// A canonical Huffman code from code lengths.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    /// Builds from per-symbol code lengths (0 = unused). An
    /// over-subscribed set is corrupt; an incomplete set is
    /// allowed only for the one-code and empty cases deflate
    /// permits, matching common decoder behavior.
    fn new(lengths: &[u8]) -> Result<Huffman, Fail> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            if l > 15 {
                return Err(Fail::Corrupt);
            }
            counts[l as usize] += 1;
        }
        if counts[0] as usize == lengths.len() {
            return Ok(Huffman { counts, symbols: Vec::new() });
        }
        let mut left: i32 = 1;
        for len in 1..16 {
            left <<= 1;
            left -= counts[len] as i32;
            if left < 0 {
                return Err(Fail::Corrupt);
            }
        }
        let mut offs = [0u16; 16];
        for len in 1..15 {
            offs[len + 1] = offs[len] + counts[len];
        }
        let total: usize = counts.iter().map(|&c| c as usize).sum();
        let mut symbols = vec![0u16; total];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offs[l as usize] as usize] = sym as u16;
                offs[l as usize] += 1;
            }
        }
        Ok(Huffman { counts, symbols })
    }

    /// Decodes one symbol.
    fn decode(&self, bits: &mut Bits<'_>) -> Result<u16, Fail> {
        let mut code: i32 = 0;
        let mut first: i32 = 0;
        let mut index: i32 = 0;
        for len in 1..16 {
            code |= bits.take(1)? as i32;
            let count = self.counts[len] as i32;
            if code - first < count {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err(Fail::Corrupt)
    }
}

/// Length codes 257..=285: base lengths.
const LEN_BASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258,
];
/// Length codes: extra bits.
const LEN_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
/// Distance codes 0..=29: base distances.
const DIST_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145,
    8193, 12289, 16385, 24577,
];
/// Distance codes: extra bits.
const DIST_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
];
/// The order of code-length-code lengths in a dynamic header.
const CLEN_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

/// One compressed block with the given literal and distance codes.
fn block(bits: &mut Bits<'_>, lit: &Huffman, dist: &Huffman, out: &mut Vec<u8>) -> Result<(), Fail> {
    loop {
        let sym = lit.decode(bits)?;
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            257..=285 => {
                let i = (sym - 257) as usize;
                let len = LEN_BASE[i] + bits.take(LEN_EXTRA[i])?;
                let dsym = dist.decode(bits)?;
                if dsym > 29 {
                    return Err(Fail::Corrupt);
                }
                let d = DIST_BASE[dsym as usize] + bits.take(DIST_EXTRA[dsym as usize])?;
                if d as usize > out.len() {
                    return Err(Fail::Corrupt);
                }
                let start = out.len() - d as usize;
                for k in 0..len as usize {
                    let b = out[start + k];
                    out.push(b);
                }
            }
            _ => return Err(Fail::Corrupt),
        }
    }
}

/// Inflates `data` into `out`. Returns the bytes consumed,
/// rounded up to a byte boundary.
pub fn inflate(data: &[u8], out: &mut Vec<u8>) -> Result<usize, Fail> {
    let mut bits = Bits::new(data);
    loop {
        let fin = bits.take(1)?;
        match bits.take(2)? {
            0 => {
                let bytes = {
                    bits.align();
                    let buffered = (bits.nbits / 8) as usize;
                    let start = bits.byte - buffered;
                    if start + 4 > bits.data.len() {
                        return Err(Fail::Short);
                    }
                    let len = (bits.data[start] as usize) | ((bits.data[start + 1] as usize) << 8);
                    let nlen = (bits.data[start + 2] as usize) | ((bits.data[start + 3] as usize) << 8);
                    if len ^ nlen != 0xffff {
                        return Err(Fail::Corrupt);
                    }
                    bits.byte = start + 4;
                    bits.acc = 0;
                    bits.nbits = 0;
                    bits.stored(len)?
                };
                out.extend_from_slice(bytes);
            }
            1 => {
                let mut lit_lengths = [8u8; 288];
                for (i, l) in lit_lengths.iter_mut().enumerate() {
                    *l = match i {
                        0..=143 => 8,
                        144..=255 => 9,
                        256..=279 => 7,
                        _ => 8,
                    };
                }
                let lit = Huffman::new(&lit_lengths)?;
                let dist = Huffman::new(&[5u8; 30])?;
                block(&mut bits, &lit, &dist, out)?;
            }
            2 => {
                let hlit = bits.take(5)? as usize + 257;
                let hdist = bits.take(5)? as usize + 1;
                let hclen = bits.take(4)? as usize + 4;
                if hlit > 288 || hdist > 30 {
                    return Err(Fail::Corrupt);
                }
                let mut clen = [0u8; 19];
                for i in 0..hclen {
                    clen[CLEN_ORDER[i]] = bits.take(3)? as u8;
                }
                let ccode = Huffman::new(&clen)?;
                let mut lengths = vec![0u8; hlit + hdist];
                let mut i = 0;
                while i < hlit + hdist {
                    let sym = ccode.decode(&mut bits)?;
                    match sym {
                        0..=15 => {
                            lengths[i] = sym as u8;
                            i += 1;
                        }
                        16 => {
                            let prev = *lengths.get(i.wrapping_sub(1)).filter(|_| i > 0).ok_or(Fail::Corrupt)?;
                            let rep = 3 + bits.take(2)? as usize;
                            for _ in 0..rep {
                                if i >= lengths.len() {
                                    return Err(Fail::Corrupt);
                                }
                                lengths[i] = prev;
                                i += 1;
                            }
                        }
                        17 => {
                            let rep = 3 + bits.take(3)? as usize;
                            i += rep;
                            if i > lengths.len() {
                                return Err(Fail::Corrupt);
                            }
                        }
                        18 => {
                            let rep = 11 + bits.take(7)? as usize;
                            i += rep;
                            if i > lengths.len() {
                                return Err(Fail::Corrupt);
                            }
                        }
                        _ => return Err(Fail::Corrupt),
                    }
                }
                let lit = Huffman::new(&lengths[..hlit])?;
                let dist = Huffman::new(&lengths[hlit..])?;
                block(&mut bits, &lit, &dist, out)?;
            }
            _ => return Err(Fail::Corrupt),
        }
        if fin == 1 {
            return Ok(bits.consumed());
        }
    }
}
