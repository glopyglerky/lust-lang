// Captures bind the variable in scope at the closure, not a later shadow.
fn main() {
    let x = 1u32;
    let f = || x * 100;
    let x = 2u32;
    println!("{} {}", f(), x);
    let mut sum = 0u32;
    let mut i = 0u32;
    while i < 3 {
        let add_i = |s: u32| s + i;
        sum = add_i(sum);
        i += 1;
    }
    println!("{}", sum);
    let words = ["a", "bb", "ccc"];
    let longest = || {
        let mut best = 0;
        for w in words {
            if w.len() > best {
                best = w.len();
            }
        }
        best
    };
    println!("{}", longest());
}
