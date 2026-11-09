// str::find and str::contains with a &str pattern (lust_str_find and lust_str_contains, which
// are lust_memmem): the leftmost match; an empty pattern (Some(0), true); a pattern longer
// than the haystack; overlapping and repeated-byte patterns; UTF-8; and haystacks of 0-71
// bytes, so both sides of memmem's 16-start blocks and its last block, which overlaps the
// block before.
fn lcg(s: &mut u32) -> u32 {
    *s = s.wrapping_mul(1103515245).wrapping_add(12345);
    *s >> 16
}

fn show(r: Option<usize>) -> String {
    match r {
        Some(i) => i.to_string(),
        None => "-".to_string(),
    }
}

fn main() {
    let fixed = [
        ("", ""),
        ("abc", ""),
        ("", "a"),
        ("ab", "abc"),
        ("aaaa", "aa"),
        ("abababab", "abab"),
        ("xxabcabc", "abc"),
        ("héllo wörld", "ö"),
        ("日本語の日本", "日本"),
        ("a/b/./c//d", "/."),
        ("a/b/./c//d", "//"),
        ("abcabd", "abd"),
        ("aaaaaaaaaaaaaaaaaaaaaaaaaaaaab", "aab"),
        ("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbba", "ba"),
    ];
    for (h, p) in fixed {
        println!("{h:?}.find({p:?}) = {}, contains {}", show(h.find(p)), h.contains(p));
    }
    let alpha = ['a', 'b', '/', '.'];
    let needles = ["a", "/.", "ab", "b.", "..", "aba", "a/b", "./a", "abab/", "/./././", "bbbbbbbbbbbbbbbbb"];
    let mut seed = 7u32;
    for hlen in 0..72usize {
        let h: String = (0..hlen).map(|_| alpha[(lcg(&mut seed) % 4) as usize]).collect();
        let mut line = format!("{hlen:2}:");
        for p in needles {
            line.push_str(&format!(" {}{}", show(h.find(p)), if h.contains(p) { "+" } else { "" }));
        }
        // Needles cut from the haystack: at its middle and at its last start.
        for plen in [2usize, 3, 5, 17] {
            if plen <= hlen {
                for at in [hlen / 2, hlen - plen] {
                    let at = at.min(hlen - plen);
                    let p = &h[at..at + plen];
                    line.push_str(&format!(" {plen}@{at}:{}", show(h.find(p))));
                }
            }
        }
        println!("{line}");
    }
}
