// Range<char> and RangeInclusive<char>: core Step for char skips the surrogates (R28).
fn show(o: Option<char>) -> u32 { o.map(|c| c as u32).unwrap_or(0xFFFF_FFFF) }
fn main() {
    let a = '\u{D7FE}'; let b = '\u{E001}';
    let v: Vec<u32> = (a..=b).map(|c| c as u32).collect();
    println!("{:x?}", v);
    let v: Vec<u32> = (a..b).rev().map(|c| c as u32).collect();
    println!("{:x?}", v);
    println!("{} {}", (a..b).count(), (a..=b).count());
    println!("{:?} {:?}", (a..b).size_hint(), (a..=b).size_hint());
    println!("{:?}", ('\0'..=char::MAX).size_hint());
    println!("{}", ('\0'..=char::MAX).count());
    for n in [0usize, 1, 2, 3, 4, 100, usize::MAX] {
        let mut r = a..b; let x = r.nth(n); print!("{:x} {:x}..{:x} | ", show(x), r.start as u32, r.end as u32);
        let mut r = (a..b).rev(); let x = r.nth(n); print!("{:x} {:x} {:x} | ", show(x), show(r.next()), show(r.next_back()));
        let mut r = a..=b; let x = r.nth(n); print!("{:x} {:x}..={:x} {} | ", show(x), *r.start() as u32, *r.end() as u32, r.is_empty());
        let mut r = (a..=b).rev(); let x = r.nth(n); println!("{:x} {:x} {:x}", show(x), show(r.next()), show(r.next_back()));
    }
    let mut r = char::MAX..=char::MAX; let x = r.next(); println!("{:?} {:?} {} {}", x, r.start(), r.is_empty(), r.contains(&char::MAX));
    let mut r = '\0'..='\0'; let x = r.next_back(); println!("{:?} {:?} {} {}", x, r.end(), r.is_empty(), r.contains(&'\0'));
    let mut r = 'x'..='\u{10FFFE}'; let x = r.nth(0x10FFFE); println!("{:?} {:x} {}", x, *r.start() as u32, r.is_empty());
    let mut r = 'x'..=char::MAX; let x = r.nth(0x10FFFF - 'x' as usize - 0x800); println!("{:?} {:x} {}", x, *r.start() as u32, r.is_empty());
    let mut r = 'x'..=char::MAX; let x = r.nth(0x10FFFF - 'x' as usize - 0x801); println!("{:?} {:x} {} {:?}", x, *r.start() as u32, r.is_empty(), r.next());
    let mut r = ('\0'..='z').rev(); let x = r.nth(200); println!("{:?} {:?} {:?}", x, r.next(), r.next_back());
    let mut r = ('\0'..='z').rev(); let x = r.nth('z' as usize); println!("{:?} {:?} {:?}", x, r.next(), r.next_back());
    println!("{:?} {:?}", ('a'..'e').last(), ('a'..='e').last());
    println!("{:?} {:?}", ('a'..'e').max(), ('a'..='e').min());
    let s: String = ('a'..='z').step_by(3).collect(); println!("{s}");
    let s: String = ('a'..='z').rev().skip(2).take(5).collect(); println!("{s}");
    let s: String = ('a'..'z').filter(|c| "aeiou".contains(*c)).collect(); println!("{s}");
    let z: Vec<(char, i32)> = ('a'..'d').zip(1..10).collect(); println!("{z:?}");
    let e: Vec<(usize, char)> = ('x'..='z').enumerate().collect(); println!("{e:?}");
    println!("{}", ('a'..='z').any(|c| c == 'q'));
    println!("{:?}", ('a'..='z').position(|c| c == 'q'));
    let mut r = 'a'..'f'; r.next(); r.next_back(); println!("{:?} {:?} {}", r.start, r.end, r.contains(&'b'));
    println!("{}", ('a'..'a').is_empty());
    let mut n = 0; for c in '\u{D7F0}'..'\u{E010}' { n += c as u32 & 1; } println!("{n}");
}
