// `unwrap` and `expect` of an Option or Result of 9 to 16 bytes aligned under 8 (the
// `Result<[u8; 8], _>` of a slice `try_into`, which x86-64 passes in two eightbytes that cut
// through its fields): release C reads the payload in place and calls the fn only on the panic
// path. Payloads, both panic messages, and a hash loop that reads 8-byte words.
use std::convert::TryInto;
use std::hint::black_box;

fn caught(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
    match std::panic::catch_unwind(f) {
        Ok(()) => "no panic".into(),
        Err(p) => match p.downcast::<String>() {
            Ok(s) => *s,
            Err(p) => match p.downcast::<&str>() {
                Ok(s) => s.to_string(),
                Err(_) => "?".into(),
            },
        },
    }
}

fn read_u64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("4 bytes"))
}

fn hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0;
    while i + 8 <= bytes.len() {
        h = (h ^ read_u64(bytes, i)).wrapping_mul(0x0100_0000_01b3);
        i += 8;
    }
    while i + 4 <= bytes.len() {
        h = (h ^ read_u32(bytes, i) as u64).wrapping_mul(0x0100_0000_01b3);
        i += 4;
    }
    for &b in &bytes[i..] {
        h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn main() {
    let data: Vec<u8> = (0..64u32).map(|i| (i * 37 + 11) as u8).collect();
    println!("{:x} {:x} {:x}", read_u64(&data, 0), read_u64(&data, 3), read_u64(&data, 56));
    println!("{:x} {:x}", read_u32(&data, 1), read_u32(&data, 60));
    for n in [0, 1, 7, 8, 9, 15, 16, 17, 31, 64] {
        println!("hash {n}: {:x}", hash(&data[..n]));
    }
    let a12: [u8; 12] = data[4..16].try_into().unwrap();
    let a15: [u8; 15] = data[..15].try_into().expect("15 bytes");
    println!("{:?} {}", a12, a15.iter().map(|&b| b as u32).sum::<u32>());
    let o: Option<[u8; 8]> = black_box(Some([1, 2, 3, 4, 5, 6, 7, 8]));
    println!("{:?} {:?}", o.unwrap(), black_box(Some([9u8; 8])).expect("some"));
    let r: Result<(u32, [u8; 4]), ()> = black_box(Ok((7, [1, 2, 3, 4])));
    println!("{:?}", r.unwrap());
    let short = black_box(&data[..5]);
    println!("{}", caught(|| println!("{:?}", <[u8; 8]>::try_from(short).unwrap())));
    println!("{}", caught(|| println!("{:?}", <[u8; 8]>::try_from(short).expect("need 8 bytes"))));
    println!("{}", caught(|| println!("{:?}", black_box(None::<[u8; 8]>).unwrap())));
    println!("{}", caught(|| println!("{:?}", black_box(None::<[u8; 8]>).expect("no bytes"))));
    println!("{}", caught(|| println!("{:?}", black_box(Err::<(u32, [u8; 4]), ()>(())).unwrap())));
}
