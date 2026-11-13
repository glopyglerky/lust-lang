// Width and precision of StripPrefixError and Path::display
// (fz2-platform): both pad and truncate the whole text as in rustc.
// The lossy non-UTF-8 form runs when LUST_BAD is set in the
// environment (the hunt generator sets it); otherwise one fixed
// line prints.
fn main() {
    let s = std::path::Path::new("a/b").strip_prefix("c").unwrap_err();
    println!("[{:>20}] [{:.3}] [{:<20}]", s, s, s);
    println!("[{:>10}] [{:.2}]", std::path::Path::new("").display(), std::path::Path::new("").display());
    println!("[{:>10}] [{:.2}]", std::path::Path::new("abc").display(), std::path::Path::new("abc").display());
    println!("[{:^9}] [{:.0}]", std::path::Path::new("a/b").display(), std::path::Path::new("a/b").display());
    match std::env::var_os("LUST_BAD") {
        Some(o) => {
            let p = std::path::Path::new(&o);
            println!("[{:>14}] [{:.3}] [{:<14}]", p.display(), p.display(), p.display());
        }
        None => println!("unset"),
    }
}
